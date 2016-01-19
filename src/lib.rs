pub mod memory;

mod registers;
mod pipeline;                   // TODO
mod barrel_shifter;             // TODO

// TODO: ALU

// TODO: MAC

// TODO: Address register

// TODO: Incrementer

// TODO: Instruction decoder

// TODO: Sign extender




#[cfg(test)]
mod test {
    // TODO: verify that a new RegisterFile starts in supervisor mode
    // and using the ARM IS

    // TODO: verify that after writing to a CPSR's mode, reads match
    // the write, and that no write leads to a None read.

    #[test]
    fn it_works() {
        assert_eq!(2,2);
    }
}
