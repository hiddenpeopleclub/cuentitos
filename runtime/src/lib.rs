use cuentitos_common::*;

pub struct Runtime {
    pub database: Database,
    running: bool,
    program_counter: usize,
    previous_program_counter: usize,
}

impl Runtime {
    pub fn new(database: Database) -> Self {
        Self {
            database,
            running: false,
            program_counter: 0,
            previous_program_counter: 0,
        }
    }

    pub fn run(&mut self) {
        self.running = true;
        self.program_counter = 0;
    }

    pub fn stop(&mut self) {
        self.running = false;
        self.program_counter = 0;
    }

    pub fn running(&self) -> bool {
        self.running
    }

    pub fn can_continue(&self) -> bool {
        self.running && !self.has_ended()
    }

    pub fn has_ended(&self) -> bool {
        self.current_block() == Some(Block::End)
    }

    pub fn current_blocks(&self) -> Vec<Block> {
        self.database.blocks[self.previous_program_counter..=self.program_counter].to_vec()
    }

    pub fn current_block(&self) -> Option<Block> {
        if self.running() {
            if self.database.blocks.len() < self.program_counter {
                None
            } else {
                Some(self.database.blocks[self.program_counter].clone())
            }
        } else {
            None
        }
    }

    pub fn step(&mut self) -> bool {
        if self.can_continue() {
            self.program_counter += 1;
            self.previous_program_counter = self.program_counter;
            return true;
        }
        false
    }

    pub fn skip(&mut self) -> bool {
        let previous_program_counter = self.program_counter;

        while self.can_continue() {
            self.step();
        }

        self.previous_program_counter = previous_program_counter;

        true
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cuentitos_common::test_case::TestCase;

    #[test]
    fn accepts_database() {
        let database = cuentitos_common::Database::default();
        let runtime = Runtime::new(database.clone());
        assert_eq!(runtime.database, database);
    }

    #[test]
    fn run_initiates_runtime() {
        let database = cuentitos_common::Database::default();
        let mut runtime = Runtime::new(database.clone());

        assert_eq!(runtime.running(), false);

        runtime.run();

        assert_eq!(runtime.running(), true);
    }

    #[test]
    fn get_current_block() {
        let test_case = TestCase::from_string(
            include_str!("../../compatibility-tests/00000000002-two-lines-and-end.md"),
            "00000000002-two-lines-and-end.md",
        );

        let database = cuentitos_parser::parse(&test_case.script).unwrap();

        let mut runtime = Runtime::new(database);

        assert_eq!(runtime.current_block(), None);

        runtime.run();

        assert_eq!(runtime.current_block(), Some(Block::Start));
        assert_eq!(runtime.current_blocks(), vec![Block::Start]);

        runtime.step();

        if let Some(Block::String(id)) = runtime.current_block() {
            assert_eq!(runtime.database.strings[id], "This is a single line");
        } else {
            assert!(
                false,
                "Expected 'This is a single line' to be returned in runtime."
            );
        }

        runtime.step();

        if let Some(Block::String(id)) = runtime.current_block() {
            assert_eq!(runtime.database.strings[id], "This is another line of text");
        } else {
            assert!(
                false,
                "Expected 'This is another line of text' to be returned in runtime."
            );
        }

        runtime.step();

        assert_eq!(runtime.current_block(), Some(Block::End));
    }
    #[test]
    fn step_moves_to_next_line() {
        let test_case = TestCase::from_string(
            include_str!("../../compatibility-tests/00000000002-two-lines-and-end.md"),
            "00000000002-two-lines-and-end.md",
        );

        let database = cuentitos_parser::parse(&test_case.script).unwrap();

        let mut runtime = Runtime::new(database);

        runtime.run();

        runtime.step();

        runtime.step();

        assert_eq!(runtime.program_counter, 2);

        if let Some(Block::String(id)) = runtime.current_block() {
            assert_eq!(runtime.database.strings[id], "This is another line of text");
        } else {
            assert!(
                false,
                "Expected 'This is another line of text' to be returned in runtime."
            );
        }
    }

    #[test]
    fn skip_moves_to_end() {
        let test_case = TestCase::from_string(
            include_str!("../../compatibility-tests/00000000003-two-lines-and-skip.md"),
            "00000000003-two-lines-and-skip.md",
        );

        let database = cuentitos_parser::parse(&test_case.script).unwrap();

        let mut runtime = Runtime::new(database);

        runtime.run();

        runtime.skip();

        assert_eq!(runtime.can_continue(), false);

        assert_eq!(runtime.current_block(), Some(Block::End));
    }

    #[test]
    fn skip_and_current_blocks_show_intermediate_blocks() {
        let test_case = TestCase::from_string(
            include_str!("../../compatibility-tests/00000000003-two-lines-and-skip.md"),
            "00000000003-two-lines-and-skip.md",
        );

        let database = cuentitos_parser::parse(&test_case.script).unwrap();
        let blocks = database.blocks.clone();

        let mut runtime = Runtime::new(database);

        runtime.run();

        runtime.skip();

        assert_eq!(runtime.can_continue(), false);

        assert_eq!(runtime.current_blocks(), blocks);
    }

    #[test]
    fn can_continue_and_has_ended() {
        let test_case = TestCase::from_string(
            include_str!("../../compatibility-tests/00000000002-two-lines-and-end.md"),
            "00000000002-two-lines-and-end.md",
        );

        let database = cuentitos_parser::parse(&test_case.script).unwrap();

        let mut runtime = Runtime::new(database);

        // Should not be able to continue if the runtime is not running.
        assert_eq!(runtime.can_continue(), false);

        runtime.run();

        // Should be able to continue if the runtime is running and has not reached
        // the end of the script.
        assert_eq!(runtime.can_continue(), true);

        runtime.step(); // step to first string
        assert_eq!(runtime.can_continue(), true);

        runtime.step(); // step to second string
        assert_eq!(runtime.can_continue(), true);

        runtime.step(); // step to end

        // Should not be able to continue if the runtime is running and has reached
        // the end of the script.
        dbg!(runtime.current_block());
        assert_eq!(runtime.has_ended(), true);
        assert_eq!(runtime.can_continue(), false);
    }

    #[test]
    fn stop_finishes_running() {
        let database = cuentitos_common::Database::default();
        let mut runtime = Runtime::new(database.clone());

        assert_eq!(runtime.running(), false);

        runtime.run();

        assert_eq!(runtime.running(), true);

        runtime.stop();

        assert_eq!(runtime.running(), false);
    }
}
