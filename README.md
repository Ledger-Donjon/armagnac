# Armagnac

Armagnac is a simple ARM emulation library written in Rust which can be used to emulate simple embedded systems. The library gives high control on the processor execution, allowing to run instruction by instruction, create hooks, inspect or modify the system state on the fly. Integration of custom peripherals in the memory space is made easy, allowing custom platforms emulation. This library has little dependencies.

The library is in development and is highly experimental. Is it not complete as not all instructions have been implemented yet. Only ARMv7M is implemented at the moment. Expect bugs, rage and frustration.

Currently, emulation is relatively slow, typically 1 million instruction per second. There is no virtualization or translation to native code whatsoever. Also, there is no "unsafe" code.
