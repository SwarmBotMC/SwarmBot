use crate::pathfind::progress_checker::ProgressChecker;

struct Pathfinding<'a> {
    progress_checker: ProgressChecker<'a>,
}

impl Pathfinding<'_> {
    fn new(progress_checker: ProgressChecker<'_>) -> Pathfinding<'_> {
        Pathfinding {
            progress_checker
        }
    }

    // fn nearby(&self) {
    //     self.progress_checker.
    // }

    // fn pf(&self){
    //     self.location
    // }
}
