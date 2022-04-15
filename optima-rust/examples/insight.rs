use optima_rust::{
    analysis::{AsCsvRow, CsvSaver},
    annealing::{
        self,
        coolers::QuadraticCooler,
        stop::{MaxSteps, NotGettingBetter},
        SimulatedAnnealing,
    },
    base::{
        solution_attr, Criterion, DerivedSolution, Evaluation, OptAlgorithm, Problem, Solution,
    },
    genetic::selection::{roulette, tournament},
};
use rand::{
    distributions::Uniform,
    prelude::{Distribution, ThreadRng},
    random, thread_rng, Rng,
};

pub type CrossFn<S> = dyn Fn(&S, &S, &mut ThreadRng) -> Vec<S>;

#[solution_attr]
#[derive(Clone, Debug, DerivedSolution)]
pub struct KnapsackSolution {
    pub picked_items: Vec<bool>,
}

impl KnapsackSolution {
    pub fn new(picked_items: Vec<bool>) -> Self {
        Self {
            picked_items,
            eval: Evaluation::default(),
        }
    }
    pub fn random_init(_id: u32, num_items: usize) -> Self {
        let mut picked_items = Vec::with_capacity(num_items);
        for _ in 0..picked_items.capacity() {
            picked_items.push(random::<bool>());
        }
        Self {
            picked_items,
            eval: Evaluation::default(),
        }
    }
}

pub fn value(problem: &KnapsackProblem, current: &KnapsackSolution) -> f64 {
    let mut total_value = 0.0;
    for i in 0..current.picked_items.len() {
        let bool_as_number: i8 = current.picked_items[i].into();
        total_value += bool_as_number as f64 * problem.values[i];
    }
    total_value
}

pub fn penalty(problem: &KnapsackProblem, current: &KnapsackSolution) -> f64 {
    let mut total_weight = 0.0;
    for i in 0..current.picked_items.len() {
        let bool_as_number: i8 = current.picked_items[i].into();
        total_weight += bool_as_number as f64 * problem.weights[i];
    }
    if total_weight > problem.capacity {
        problem.capacity - total_weight
    } else {
        0.0
    }
}

fn change_solution(solution: &mut KnapsackSolution, _problem: &KnapsackProblem) {
    let mut rng = thread_rng();
    let random_index: usize = rng.gen_range(0..solution.picked_items.len());
    solution.picked_items[random_index] = !solution.picked_items[random_index];
}

fn change_population(population: &mut Vec<KnapsackSolution>, rng: &mut ThreadRng) {
    let uniform = Uniform::new(0, population.len());
    let mut children = Vec::with_capacity(population.len());

    while children.len() < population.len() {
        let father = &population[uniform.sample(rng)];
        let mather = &population[uniform.sample(rng)];

        let cross_point = rng.gen_range(1..father.picked_items.len());

        let from_father = father.picked_items[..cross_point].to_vec();
        let from_mather = mather.picked_items[cross_point..].to_vec();

        let child_a_picked_items = vec![from_father, from_mather].concat();

        let from_mather = mather.picked_items[..cross_point].to_vec();
        let from_father = father.picked_items[cross_point..].to_vec();

        let child_b_picked_items = vec![from_mather, from_father].concat();

        let child_a = KnapsackSolution::new(child_a_picked_items);
        let child_b = KnapsackSolution::new(child_b_picked_items);

        children.push(child_a);
        children.push(child_b);
    }

    let mutate_rate = 0.5;
    for i in 0..population.len() {
        let child = &mut children[i];
        for j in 0..child.picked_items.len() {
            if random::<f64>() < mutate_rate {
                child.picked_items[j] = !child.picked_items[j];
            }
        }
        population[i] = child.clone();
    }
}

fn random_population(size: usize, num_items: usize) -> Vec<KnapsackSolution> {
    let mut population = Vec::with_capacity(size);
    for i in 0..size {
        let specimen = KnapsackSolution::random_init(i as u32, num_items);
        population.push(specimen);
    }

    population
}
#[derive(Clone, Copy)]
pub struct KnapsackProblem<'a> {
    id: u32,
    pub weights: &'a Vec<f64>,
    pub values: &'a Vec<f64>,
    pub capacity: f64,
}
impl<'a> KnapsackProblem<'a> {
    pub fn new(id: u32, weights: &'a Vec<f64>, values: &'a Vec<f64>, capacity: f64) -> Self {
        Self {
            id,
            weights,
            values,
            capacity,
        }
    }
}
impl<'a> Problem for KnapsackProblem<'a> {
    fn get_id(&self) -> u32 {
        self.id
    }
}
impl AsCsvRow for KnapsackSolution {
    fn as_row(&self, i: usize) -> String {
        format!("{},{}", i, self.get_value())
    }
}

fn main() {
    let weights = vec![1.0, 2.0, 3.0, 8.0, 12.0, 20.0, 30.0];
    let values = vec![4.0, 5.0, 1.0, 2.0, 8.0, 5.0, 6.0];
    let capacity = 6.0;

    let initial_solution = KnapsackSolution::random_init(0, weights.len());
    let problem1 = KnapsackProblem::new(0, &weights, &values, capacity);
    let problem2 = KnapsackProblem::new(1, &weights, &values, capacity);

    let mut criterion = Criterion::new(&penalty, &value, false);
    let cooler = QuadraticCooler::new(1000.0, 0.997);
    let max_steps = MaxSteps::new(20000);

    let header = String::from("iter,value");
    let mut prev_problem_id = u32::MAX;
    let mut csv = CsvSaver::new(String::from("./0.csv"), header);
    let mut insight = move |_: u32,
                            problem: &KnapsackProblem,
                            best: &KnapsackSolution,
                            _: &KnapsackSolution,
                            last_call: bool| {
        if last_call {
            csv.flush();
            return;
        }

        let id = problem.get_id();
        if id != prev_problem_id {
            csv.flush();
            let file = format!("./{}.csv", problem.get_id());
            csv.reset(file, None);
        }
        prev_problem_id = id;
        csv.save_element(best);
    };

    let mut annealing = SimulatedAnnealing::new(
        &initial_solution,
        max_steps,
        cooler,
        &change_solution,
        &mut insight,
    );

    let mut solutions = vec![];

    solutions.push(annealing.solve(problem1, &mut criterion));
    println!();
    solutions.push(annealing.solve(problem2, &mut criterion));
}