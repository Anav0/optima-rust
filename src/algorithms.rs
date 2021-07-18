use crate::{
    base::{Solution, State, StateChanger},
    coolers::Cooler,
    criterion::Criterion,
    stop::StopCriteria,
};
use rand::{prelude::ThreadRng, Rng};
use std::f64::consts::E;

pub trait OptAlghorithm<'a, S>
where
    S: Solution,
{
    type SolutionType;

    fn solve(
        &mut self,
        solution: &mut S,
        criterion: &mut Criterion<S>,
        mover: &'a mut dyn StateChanger<SolutionType = Self::SolutionType>,
    );
}

pub struct SimmulatedAnnealing<'a> {
    stop_criteria: &'a mut dyn StopCriteria,
    cooler: &'a mut dyn Cooler,
    rnd: ThreadRng,
}

impl<'a> SimmulatedAnnealing<'a> {
    pub fn new(stop_criteria: &'a mut dyn StopCriteria, cooler: &'a mut dyn Cooler) -> Self {
        Self {
            stop_criteria,
            cooler,
            rnd: rand::thread_rng(),
        }
    }
    fn hot_enought_to_swap(&mut self, solution: &dyn Solution) -> bool {
        let diff =
            solution.get_info(State::Current).value - solution.get_info(State::BeforeChange).value;

        if diff == 0.0 {
            return false;
        };

        if diff > 0.0 {
            return true;
        }

        return self.rnd.gen::<f64>() < E.powf(diff / self.cooler.get_temp());
    }
}

impl<'a, S> OptAlghorithm<'a, S> for SimmulatedAnnealing<'a>
where
    S: Solution,
{
    type SolutionType = S;
    fn solve(
        &mut self,
        solution: &mut S,
        criterion: &mut Criterion<S>,
        state_changer: &'a mut dyn StateChanger<SolutionType = Self::SolutionType>,
    ) {
        let mut iter: u32 = 0;

        solution.reset();
        criterion.initial(solution);

        while !self.stop_criteria.should_stop(solution) {
            solution.replace_before();
            state_changer.change_state(solution);
            solution.set_info(f64::NAN, false, true);

            if criterion.is_first_better(solution, State::Current, State::BeforeChange)
                || self.hot_enought_to_swap(solution)
            {
                if criterion.is_first_better(solution, State::Current, State::Best) {
                    solution.replace_best();
                }
            } else {
                solution.replace_current();
            }

            self.cooler.cool();
            iter += 1;
        }
    }
}
