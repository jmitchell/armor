// TODO: Deal with incorrect assumption that each instruction takes
// one cycle per stage in the pipeline. (How does this even work? Do
// multi-cycle/stage instructions just hold up the preceding stages
// while the following ones proceed as normal? See ASDG pp165)

// TODO: Model pipeline hazards (aka interlock) and the resulting
// "bubbles".

trait Pipeline {
    fn run_cycle(&self) {
        // TODO: Check for interrupt? (And if not here, where?)

        self.set_fill({
            if self.filled_to() < self.stage_count() {
                self.filled_to() + 1
            } else {
                self.filled_to()
            }
        });

        for i in 0..self.filled_to() {
            // TODO: Ensure there's no interference between the
            // stages. Ways to get around that might include doing
            // work synchronously from the last stage to the first or
            // copying state from the CPU needed for each stage before
            // any work begins (and optionally doing it concurrently).
            self.run_stage(i);
        }
    }

    fn stage_count(&self) -> usize;

    fn run_stage(&self, usize);

    fn filled_to(&self) -> usize;

    fn set_fill(&self, usize);

    fn flush(&self) {
        self.set_fill(0);

        // TODO: Does a flush only occur after branches and
        // interrupts? What portion of the pipeline is cleared, and
        // what parts are allowed to continue executing?
    }
}

trait ThreeStagePipeline : Pipeline {
    fn stage_count(&self) -> usize {
        3
    }

    fn run_stage(&self, index: usize) {
        match index {
            0 => self.fetch_stage(),
            1 => self.decode_stage(),
            2 => self.execute_stage(),
            _ => panic!("Invalid stage index for 3-stage pipeline"),
        }
    }

    fn fetch_stage(&self);
    fn decode_stage(&self);
    fn execute_stage(&self);
}

trait FiveStagePipeline : Pipeline {
    fn stage_count(&self) -> usize {
        5
    }

    fn run_stage(&self, index: usize) {
        match index {
            0 => self.fetch_stage(),
            1 => self.decode_stage(),
            2 => self.execute_stage(),
            3 => self.memory_stage(),
            4 => self.write_stage(),
            _ => panic!("Invalid stage index for 3-stage pipeline"),
        }
    }

    fn fetch_stage(&self);
    fn decode_stage(&self);
    fn execute_stage(&self);
    fn memory_stage(&self);
    fn write_stage(&self);
}

trait SixStagePipeline : Pipeline {
    fn stage_count(&self) -> usize {
        6
    }

    fn run_stage(&self, index: usize) {
        match index {
            0 => self.fetch_stage(),
            1 => self.issue_stage(),
            2 => self.decode_stage(),
            3 => self.execute_stage(),
            4 => self.memory_stage(),
            5 => self.write_stage(),
            _ => panic!("Invalid stage index for 3-stage pipeline"),
        }
    }

    fn fetch_stage(&self);
    fn issue_stage(&self);
    fn decode_stage(&self);
    fn execute_stage(&self);
    fn memory_stage(&self);
    fn write_stage(&self);
}
