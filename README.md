Malakh
======

Malakh is an new programming language, centered around the idea of "processes". Processes are a new concept invented in Malakh, which unifies the notions of "functions" and "classes". A process can act like a function, an object, or both – or like many things that don't exist in other languages.

Example
-------

Here's a function definition in Malakh:

    Max -> {
        a, b := in
        if a >= b {
            out a
        }
        else {
            out b
        }
    }

And here's a class (named `ComplexNumber`, with methods `.RealPart` and `.ImaginaryPart`):

    ComplexNumber -> {
        real, imag := in
        loop {
            switch in {
            
            case .RealPart:
                out real

            case .ImaginaryPart:
                out imag
            
            }
        }
    }

As you can see, there's very little difference between the two definitions. Here's how you might use these definitions:

    Main -> {
        // Print the maximum of 10 and 20:
        out [Max 10 20]
        // Define i (the square root of -1):
        i := ComplexNumber 0.0 1.0
        // Print the real part and the imaginary part of i:
        out [i .RealPart] [i .ImaginaryPart]
    }

Learning
--------

There's an (incomplete) tutorial in [doc.md](doc.md). More documentation will hopefully be added soon.
For now, you can check out some examples in [./test/success](./test/success).

Running Malakh Code
-------------------

To run a Malakh program, follow these steps:
1. Download the repository and compile it using Cargo (`cargo build`).
2. Write your program in a .mal file. Your program must contain a process called `Main`.
3. Run `cargo run -- path_to_program.mal`.
   - Alternatively, run `path_to_executable path_to_program.mal`, where `path_to_executable` is something like `./target/debug/malakh.exe` (you may move or rename the executable however you like).
4. Now the `Main` process should start running. Congratulations!

The Name Malakh
---------------

In ancient Hebrew, "malakh" means a messenger – a fitting name for a language that's all about sending messages between different entities (processes).
In modern Hebrew, "malakh" means an angel.
