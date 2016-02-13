#![allow(dead_code)]

use address;
use processor;

// TODO: Support serializing and deserializing to a human-readable file.

struct Computer {
    cpu: processor::Processor,
    address_space: address::AddressSpace,
}
