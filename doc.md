About Malakh
=================
**Draft**

Malakh is a language that unifies the concepts of *Functions* and *Objects*
into a single concept called a *Process*.
Processes are a very powerful concept, giving you much expressive power,
but are based on a relatively simple, intuitive model.

What Is a Processes
-------------------
Every program in Malakh is made of processes.
A process is an entity that can:

- Read input, either from the user or from another process.
- Send input to another process.
- Write output, to be read either by the user or by another process.
- Receive the output of another process.
- Run arbitrary computations.
- Store data.
- Create more processes.

Every program in Malakh consists of
a list of the kinds of processes in it,
their behavior, and the interactions between them.

What Processes Are Not
----------------------

Processes are similar to, but not the same as, operating system processes.
Operating system processes are the programs that run on your computer &ndash;
each program is a process.
All the operating system processes often run in parallel.
On the other hand, Malakh's processes are all
parts of the same program.
They usually don't run in parallel, but one at a time.

Why Processes
-------------
Most other languages contain some weaker form of processes:

- Most language have *functions*,
  which are processes that can receive an input (called their argument),
  perform some computation, and return an output (called their return value).
- In many languages, functions can also receive multiple inputs,
  or no input at all.
- In many languages, functions can return multiple outputs,
  or no output at all.
- In some languages, functions can receive their inputs gradually,
  instead of all at once. Such functions are called *curried functions*.
- In some languages, functions can return their outputs gradually,
  instead of all at once. Such functions are called *generators*.
- Most languages have *data structures*, which are processes that can store data.
- Many languages have *objects*, which are data structures that
  can also receive input (called *messages* or *method calls*)
  and respond with output.

Most languages don't support all of these features.
Even if they do, they use different syntax for each one of them,
which makes those languages very complex.
Malakh's processes can do all of these things and many more,
with a simple, consistent syntax.

Tutorial
========

Missing Features
----------------

Before you read the tutorial, it's important that you know that
some features mentioned in the tutorial are not yet implemented.
Hopefully, they will all be added soon:

- The `this` keyword.
- The `inout` keyword.
- Local constructors.
- Accessing local variables from nested processes.
- try-finally.
- `Data::Count`.
- `MinHeap` and `MaxHeap`.
- `String::FromCharCodes`, `String::CharCodes`.

Output
------

Let's start with the famous "Hello, World!" program.
In this program, we define the `Main` process,
which is responsible for communicating with the user.
Inside `Main`, we use the `out` command to send output to the user.

    Main -> {
        out "Hello, World!"
    }

The program outputs "Hello, World!".

A program can also write multiple outputs:

    Main -> {
        out 1
        out 2
        out 3
    }

This program outputs the numbers 1, 2 and 3, each on a separate line.\
You can also write the same program like this:

    Main -> {
        out 1 2 3
    }

This program is equivalent to the above program &ndash;
it prints the numbers 1, 2 and 3, each on a separate line.

You can also output negative numbers or more complicated expressions,
but you have to wrap them in parentheses:

    Main -> {
        out (-1) (1 + 1)
        out (2*2 + 3)
    }

This program outputs the numbers -1, 2 and 7.

Comments
--------

There are two types of comments in Malakh:

- Line comments start with `//` and go until the end of the line.
- Multiline comments start with `/*` and end with `*/`:

```
    // I am a comment.
    // This program doesn't do anything!
    Main -> {
        /* I am a
           long,
           long
           comment. */
    }
```

Mathematical Operators
----------------------
Malakh has the following mathematical operators:

- `+`: add two numbers.
- `-`: subtract two numbers, or negate a number.
- `*`: multiply two numbers.
- `/`: divide two numbers.
- `%`: divide two numbers and get the remainder.
- `^`: raise `a` to the power of `b`.

For example:

    Main -> {
        out (1 + 1)    // Output: 2
        out (5 - 2*2)  // Output: 1
        out (10 - 4/2) // Output: 8
        out (5 % 3)    // Output: 2
        out (2 ^ 5)    // Output: 32
    }

Mathematical Processes
-------------------
Although you haven't really learnt about processes yet,
here are a few useful processes:

- `Sum` finds the sum of a group of numbers. For example:
    
        Main -> {
            out [Sum 1 2 3 4 5] // Output: 15
            out [Sum 12 18]     // Output: 30
            // Note that you need parentheses around negative numbers:
            out [Sum 10 (-2)]   // Output: 8
        }
- `Math::Product` is similar to `Sum`, but finds the product.
- `Math::Mean` finds the mean (average).
- `Min` and `Max` find the minimum and maximum, respectively.
- `Math::Abs` finds the absolute value of a number. For example:

        Main -> {
            out [Math::Abs (-3)]                     // Output: 3
            out [Math::Abs 5]                        // Output: 5
            out [Math::Abs [Math::Mean (-10) (-20)]] // Output: 15
        }
- `Math::Round` rounds a number to the nearest integer.
  `Math::Floor` rounds a number down, and `Math::Ceil` rounds it up.
- `Math::Sin` finds the sine of an angle (in radians).
  `Math::Cos`, `Math::Tan`, `Math::Cot`, `Math::Asin`, `Math::Acos`,
  `Math::Atan` and `Math::Acot` are similar.
- `Math::Exp` computes `e^x`.
- `[Math::Log x]` is the natural logarithm of `x`.
  `[Math::Log b x]` is the logarithm of `x` to base `b`.

Variables
---------
You can define variables with `:=`:

    Main -> {
        one := 1
        two := 2
        out (one + two)
    }
    // Output: 3

You can also define variables outside of the `Main` process.
These variables are called *global variables*,
and will be visible to all processes:

    Global := "foo"

    Main -> {
        local := "bar"
        out Global local
    }
    // Output:
    // foo
    // bar

After defining a local variable, you can change its value with `=`:

    Main -> {
        x := 1
        out x // Output: 1
        x = 2
        out x // Output: 2
        x = 3
        out x // Output: 3
    }

But you can only modify local variables, not global ones.

TODO: explain about `+=`, `-=`, `*=`, `/=`, `%=`, `^=`.

Constructors
------------
A constructor is another type of variable.
A constructor is defined like a variable, but with `->` instead of `:=`.

Unlike a variable, a constructor doesn't store its value anywhere in memory.
Instead, it only stores the expression given to it. For example, if you
define a constructor `z -> x / y`, then the name `z` will become tied to the
expression `x / y`. If you try to access `z` (for example, by running
`out z`),
the program will evaluate the expression `x / y`, and give you the result.
If you access the constructor `z` a number of times, it will evaluate `x / y`
every time; it's possible that the value of `z` will be different every time,
if `x` and `y` change, or even that the program will crash, if `y` equals 0.

For example:

    Main -> {
        x := 1        // x is a variable.
        foo -> x * 10 // foo is a constructor.
        out foo       // Output: 10
        x = 2         // change x, which will indirectly change foo.
        out foo       // Output: 20
    }

Unlike a local variable, you can't use `=` with a constructor
to change its value. You can only change its value indirectly, as shown above.

(You may have noticed that `Main` is also defined with `->`.
That's because `Main` is a constructor too!)

<!--
Naming
------
The name of a variable or a constructor may contain
english letters (both uppercase and lowercase), digits and underscores (`_`).\
The name of a local variable or constructor must start with
a lowercase letter.
The name of a global variable or constructor must start with
either an uppercase letter or a lowercase letter.
Additionally, names may start with a single underscore, followed by a letter.\
For example:

    GlobalVar := 123
    GlobalCon -> "Hello"

    Main -> {
        localVar := "foo bar"
        localCon -> "lorem ipsum"
    }

Later you'll see that global variables and constructors,
whose names start with a capital letter, are called "public",
and ones that start with a lowercase letter are called "private".
For now, we'll only use "public" globals.
-->

Data Types
----------
Every value in Malakh belongs to one of five types:

- **Integer (Int):** whole numbers like `1`, `-3`, `1000`, etc.
  Integers in Malakh may be arbitrarily large.\
  Integer literals can contain underscores to make them easier to read,
  for example, `1_000_000 = 1000000`.\
  Integers literals may also be binary or hexadecimal,
  for example, `0b_1010_0000 = 0xa0 = 160`.
- **Floating-Point Number (Float):** fractions like `1.2`, `-30.5`, `1.0`,
  and also the special values: `Infinity`, `-Infinity` and `NaN`
  ("not a number").
  Floats are stored as 64-bit IEEE-754 floating point numbers.\
  Float literals may contain underscores (e.g. `10_000.000_1`)
  and an exponent (e.g. `2.3e-5`).
- **String:** texts like `"Hello"`, `"Lorem ipsum."`, `""`, `"Ру́сский язы́к"`,
  and `"😂🤖👾"`.\
  String literals are surrounded with double quotes (`""`),
  and may contain character escapes:
  * `\n` - line feed.
  * `\r` - carriage return.
  * `\t` - tab.
  * `\"` - double quotes.
  * `\\` - backslash.
  * `\xDD` - two-digit hexadecimal byte.
  * `\uDDDD` - four-digit hexadecimal character code.
  * `\UDDDDDDDD` - eight-digit hexadecimal character code.
- **Symbol:** symbols are like one-word strings.
  Unlike strings, symbols may not (normally) contain spaces or special
  characters,
  but comparing symbols is much faster than comparing strings.\
  Symbol literals start with a dot, for example:
  `.MySymbol123`.
  Symbols are often used to represent special constants,
  like `.True`, `.False` and `.Undefined`.
  <!--Like variables and constructors, symbols can be either public or private.
  A symbol is public if the first letter in it is uppercase,
  or private otherwise.
  You'll see the difference between public and private symbols later.
  For now, we'll only use public symbols.-->
- **Process:** the building blocks of a program.
  Processes will be discussed later.\
  Process literals are surrounded by braces,
  for example: `{ out "Hello" }`.

Malakh is dynamically-typed,
so the type of each variable is not known before the program starts running,
and can change while the program is running.
You can use the processes:
`IsNumber`, `IsInt`, `IsFloat`, `IsString`, `IsSymbol`, and `IsProcess`
to determine the type of a value.
They return the symbol `.True` if their input has the expected type,
or the symbol `.False` otherwise.
(`IsNumber` expects either an Int or a Float.)
For example:

    Main -> {
        x := 123
        out [IsNumber x] // True
        out [IsInt x]    // True
        out [IsFloat x]  // False
        x = .Foobar
        out [IsNumber x] // False
        out [IsSymbol x] // True
    }

Input
-----
You can use the `in` command to receive input, for example:

    // Receive one line of input from the user, then output it twice.
    Main -> {
        x := in
        out x
        out x
    }

You can read multiple lines of input like this:

    first := in
    second := in
    third := in

Or, as a shortcut:

    first, second, third := in

Optional Input
--------------
Usually, when an `in` expression runs, it crashes if there is no input to read.
You can use the `else` keyword instead, to define an action to do
when there's no input. For example:

    Main -> {
        out "Please say something:"
        userInput := in
        else {
            out "You didn't say anything!"
            userInput = ""
        }
        // The process `String::Length` finds the length of a string.
        out [String::Length userInput]
    }

When we run this program with the input "foo", we get the following output:

    Please say something:
    [User enters: foo]
    3

When we run this program with no input, we get the following output:

    Please say something:
    You didn't say anything!
    0

(The `else` keyword has to appear immediately after a variable declaration
or assignment statement, where the value being assigned is `in`.
You can also use `else` in some other places, as we'll see later.)

Booleans
--------

The symbols `.True` and `.False` are called the "Boolean" symbols.
They are treated specially by `if` and `while` statements,
as you'll see soon. \
The process `IsBool` gets a value and outputs `.True` if it is a Boolean
symbol (i.e. it equals either `.True` or `.False`),
or `.False` otherwise.

Comparison Operators
--------------------

Malakh has the following comparison operators:

- `==`: checks if two values are equal.
  Returns `.True` if they are equal, or `.False` otherwise.
- `!=`: checks if two values are not equal.
- `<`: checks if one value is less than another.
- `>`: checks if one value is greater than another.
- `<=`: checks if one value is less than or equal to another.
- `>=`: checks if one value is greater than or equal to another.

TODO explain about behavior for each type.

For example:

    Main -> {
        out (1 == 1)             // True
        out (1 == 3 + 4)         // False
        out (2 == 1.0 + 1.0)     // True
        out ("hello" == "hello") // True
        out (1 == "1")           // False (a string is never equal to a number)
        out ((1 == 3) == .False) // True
        out (1 != 2)             // True
        out (5 != 5.0)           // False
        out (1 < 2)              // True
        out (4 >= 4)             // True
        out (4 >= 5)             // False
        out ("Cat" < "Dog")      // True (strings are compared lexicographically)
    }

Logical Operators
-----------------

Malakh has four logical operators:

- `and`: takes two booleans, and checks if they are both `.True`.
- `or`: takes two booleans, and checks if at least one of them is `.True`.
- `xor`: takes two booleans, checks if exactly one of them is `.True`.
- `not`: takes a Boolean, and yields the opposite Boolean.

For example:

    Main -> {
        out (.True and .True)   // True
        out (.True and .False)  // False
        out (.True or .False)   // True
        out (.False or .False)  // False
        out (.True xor .True)   // False
        out (.True xor .False)  // True
        out (.False xor .True)  // True
        out (.False xor .False) // False
        out (not .True)         // False
        out (not .False)        // True
    }

TODO: explain short-circuiting.

If Statement
------------

An `if` statement takes a boolean value, and runs some block of code
only if the value is `.True`:

    Main -> {
        line := in
        if line == "secret password" {
            out "Wow! You know the secret password!"
        }
    }

An `if` statement may be followed by an `else` clase.
The `else` will run only if the condition was `.False`:

    Main -> {
        out "Please enter a number:"
        // Receive input and convert it to a number.
        value := [ToNumber in]
        if value == 0 {
            out "Zero."
        }
        else {
            out "Nonzero."
        }
    }

An `else if` clause will run only if its condition is `.True`,
and the previous condition(s) were `.False`:

    Main -> {
        out "Please enter a number:"
        // Receive input and convert it to a number.
        value := [ToNumber in]
        if value < 0 {
            out "Negative."
        }
        else if value == 0 {
            out "Zero."
        }
        else {
            out "Positive."
        }
    }

While Statement
---------------

A `while` statement takes a boolean value, and runs some block of code
as long as the value is `.True`:

    // This program will keep running until the user says "please".
    Main -> {
        line := in
        while line != "please" {
            out "You didn't say please!"
            line = in
        }
        out "OK."
    }

Inside of a `while` statement, you may use `break` and `continue` statemets.

- `break` causes the loop to stop immediately.
- `continue` jumps back to the beginning of the loop and checks the condition again.
  If the condition is still `.True`, it starts another iteration of the loop;
  if it is `.False`, it stops the loop.

For example:

    // This program outputs all the numbers between 1 and 10, except 7.
    Main -> {
        i := 0
        while .True {
            i += 1
            if i == 7 { continue }
            out i
            if i == 10 { break }
        }
    }

Loop Statement
--------------

A `loop` statement begins an infinite loop.
The loop runs forever, or until it is stopped by
a `break` statement or something similar.
For example:

    // This program repeatedly reads inputs, and outputs them back to the user.
    Main -> {
        loop {
            out in
        }
    }

(`loop` is equivalent to `while .True`, but more readable.)

Stop Statement
--------------

A `stop` statement makes the current process stop running.
If `stop` appears inside the `Main` process,
it makes the whole program terminate.
For example:

    Main -> {
        line := in
        else {
            out "You didn't say anything."
            stop
        }
        out "You said:" line
        // When a process reaches the end of its code,
        // it stops, even if there's no `stop` statement.
    }

If and While Statement with Input
---------------------------------

There is a second form of `if` statement, written:
`if x := in` or `if x = in`.
This type of statement attempts to read an input.
If it succeeds, it assigns the input to a variable,
and runs the following block of code.
For example:

    Main -> {
        if line := in {
            out "You said:" line
        }
    }

Similarly, there is a second form of a `while` statement, written:
`while x := in` or `while x = in`.
This type of loop repeatedly tries to read inputs,
and runs a block of code as long as it succeeds.
For example:

    // This program reads a sequence of numbers, and outputs their sum.
    Main -> {
        sum := 0
        while n := in {
            // The `ToNumber` process converts a string to a number.
            sum += [ToNumber n]
        }
        out "The sum is:" sum
    }

Assertions
----------

The `Assert` process takes a boolean
and crashes if it is not `.True`.
This is useful if you want to make sure a certain condition holds
during your code's executing. For example:

    Main -> {
        Assert (1 + 1 == 2)
        Assert (3 + 4 == 7)
        Assert [IsNumber 1.3]
        out "All tests passed."
    }
    // Since all the conditions are true,
    // this program will output: "All tests passed."

In all the following examples, whenever you see an assertion,
you can always assume that its condition is true.

Processes
---------

Finally, it's time to talk about the most important thing
in Malakh, processes.
A process is simply a block of code &ndash; that is,
a sequence of instructions to execute.

To create a process, write some instructions in Malakh,
wrapped in braces (`{}`).
You'll often want to store the process in a variable,
to use it later.
For example:

    Main -> {
        // Create a process.
        firstProcess := {
            out 1
            out 2
        }
        // Create another process.
        secondProcess := {
            n := in
            out (n + 1)
        }
        Assert (firstProcess != secondProcess)
    }

When you create a process, it immediately starts running.
So in the above example, `firstProcess` will immediately output the value 1,
and `secondProcess` will immediately try to read input &ndash;
as soon as these processes are created.

Processes can create variables,
like `secondProcess` does in the above example.
If a process creates a variable,
that variable is local to that process,
so no process outside of it can read or write that variable.
Moreover, two different processes can define two variables with the same name.

Input and Output in Processes
-----------------------------

Unlike the `Main` process, other processes don't read their input from
the user, and don't write their output to the user either.
The meaning of the `in` and `out` commands is different than in the `Main`
process.

When a process (other than `Main`) encounters an `in` command,
it pauses and waits for someone to send it input.
This "someone" won't be the user, but rather, another process
will have to send it input.
Once the process receives its input,
It will resume running from the point where it stopped.
We'll see later how to send input to a process.

For output, each process has a special "output slot",
where it stores its outputs.
When a process (other than `Main`) encounters an `out` statement,
it takes the value it intends to output,
and places it in its output slot.
Then, it pauses until someone reads the output.
When another process reads that output,
that process empties the first process's output slot, and notifies it that it
can continue running.
Then, the first process resumes running from the point where it stopped
(immediately after the `out` command).

If a process outputs multiple values at the same time
(e.g. it executes the line: `out 1 2 3`),
the process just repeats these steps multiple times.
In this case, it outputs 1; then, once someone reads the 1,
it outputs 2; and finally, once someone reads the 2, it outputs 3.

Receive Expression
------------------

As we just saw, when a process outputs a value,
it immediately starts waiting for someone else to read it.
Now we'll see how to read the output of a process.

To read the output of a process,
another process has to run a *receive* expression.
A *receive* expression looks like this: `[otherProcess]`.
For example:

    Main -> {
        // Create a process called "counter".
        counter := {
            out 1
            out 2
            out 3
        }
        firstOutput := [counter]
        Assert (firstOutput == 1)
        secondOutput := [counter]
        Assert (secondOutput == 2)
    }

Let's look at this program, step by step:

1. First, the `Main` process creates a process called `counter`.
   `counter` is a process that outputs the numbers 1, 2 and 3.
2. `counter` starts running immediately after it's created.
   It immediately reaches the first `out` statement (`out 1`),
   and outputs the number 1. It pauses until someone reads that.
3. Once `counter` has paused, `Main` can resume running.
   `Main` runs the line `firstOutput := [counter]`.
   This causes it to read the first output of `counter` (which is 1),
   and notify `counter` that it can resume running.
4. `counter` resumes running. It executes the next line,
   `out 2`. This causes it to output 2 and pause again.
5. `Main` asserts that the first output it got equals 1.
6. `Main` runs the line `secondOutput := [counter]`.
   This causes it to read the second output of `counter` (which is 2),
   and notify `counter` that it can resume running.
7. `counter` resumes running. It executes the next line,
   `out 3`. This causes it to output 3 and pause again.
8. `Main` asserts that the second output it got equals 2.
9. `Main` stops running. This means that the whole program terminates.
   Interestingly, at the end of the program,
   `counter` was still waiting for someone
   to read its output, But no one did.
   This is fine, the program just terminates as usual.

Don't worry if the details here seem complicated.
Writing programs in Malakh tends to be fairly intuitive,
even if you don't remember all the details.

Optional Receive Expression
---------------------------

One downside of a *receive* expression is that it expects the process
it reads from to have some output.
If that process currently has no output, the receiving process simply crashes.
Luckily, there are two ways to read the output of a process,
without crashing if there is none:

First, assignment with an `else` clause:

    // `Main` tries to read the output of `proc`,
    // but shows a nice error message if it fails.
    Main -> {
        proc := { /* some process */ }
        procOutput := [proc]
        else {
            out "proc didn't output anything. Too bad."
            stop
        }
        out procOutput
    }

(You have already seen optional `in` expressions, so these are very similar.)

Secondly, you can read optional output in an `if` statement.
Again, this is very similar to what you can do with an `in` expression:

    // This program is equivalent to the previous one.
    Main -> {
        proc := { /* some process */ }
        if procOutput := [proc] {
            out procOutput
        }
        else {
            out "proc didn't output anything. Too bad."
        }
    }

*Note*: the rules about when receive expressions succeed in reading output,
and when they fail, are a bit complicated.
We'll discuss them in more depth later.

Send Expression
---------------

Just like a process can receive another process's output
with a *receive* expression,
it can also send input to a process using a *send* expression.
A *send* expression looks like this: `process value`
(that is, the process, then a space,
then the value you're sending).
For example:

    Main -> {
        // `add` is a process that gets two numbers and outputs their sum.
        add := {
            a := in
            b := in
            out (a + b)
        }
        // Send the number 3 to `add`.
        add 3
        // Send the number 4 to `add`.
        add 4
        // Receive the output of `add`, and check if it's 7.
        Assert ([add] == 7)
    }

Let's look at this program, step by step:

1. First, the `Main` process creates a process called `add`.
2. `add` starts running immediately.
   It runs the statement `a := in`,
   which causes it to pause and wait for input.
3. `Main` resumes running.
   It runs the statement `add 3`,
   which sends the value 3 to `add`.
4. Once `add` gets the input 3, it resumes running.
   It stores the input in `a`.
5. `add` runs the statement `b := in`,
   which causes it to pause and wait for input.
6. `Main` resumes running.
   It runs the statement `add 4`,
   which sends the value 4 to `add`.
4. Once `add` gets the input 4, it resumes running.
   It stores the input in `b`.
5. `add` runs the statement `out (a + b)`.
   It stores the number 7 (`a + b`) in its *output slot*,
   and waits for someone to read it.
6. `Main` resumes running, and reads the output of `add`.
7. `add` resumes running, but immediately stops,
   because it has reached the last line of its code.
8. `Main` resumes running.
   It checks if the value it has just read is 7 (which it is).
9. `Main` reaches the last line of its code,
   and the program terminates.

*Note*: a *send* expression expects the process
to which it is sending input,
to be in a state where it expects input
(i.e. it should be a process that has just executed an `in` expression).
It is an error to send input to a process that is not expecting input.

The Result of Send Expressions
------------------------------

When you execute a *send* expression, you can also use the expression's result,
like so:

    // Send `value` to `process`, and store in `var`.
    var := process value

The result of a *send* expression is the process to which it sent the value.
So in the above example, the variable `var` will get the value `process`.
The above example is equivalent to:

    process value
    var := process

There are two common cases where this is useful:

First, this means you can send multiple inputs
to a process on the same line.
For example, instead of writing:

    process 1
    process 2
    process 3

You can write:

    ((process 1) 2) 3

Or simply:

    process 1 2 3

This works because the expression `process 1` yields the process `process`
itself, which means you can immediately send it another input.
And the second *send* expression also yields `process`,
so you can immediately send it the third input.

It is also possible to write a *send* expression
inside a *receive* expression:

    // Send `value` to `process`,
    // then immediately read `process`'s output and store it in `var`.
    var := [process value]

Or even:

    // Send 1, 2 and 3 to `process`,
    // then immediately read `process`'s output and store it in `var`.
    var := [process 1 2 3]

Again, this works because a *send* expression yields back the same process.

Now is a good time to recall the mathematical processes we have seen earlier,
such as:

    Main -> {
        out [Math::Sin 3.14]  // Output: 0.00159265291648683
        out [Sum 1 2 3] // Output: 6
    }

Now you can finally understand how these work!
for example, in the expression `[Math::Sin 3.14]`,
we send the number 3.14 to the process `Math::Sin`,
and then read its output &ndash; which is the sine of 3.14.

Process Constructor
-------------------

When you define a process,
you will almost always want to define it inside a global constructor,
like so:

    MyProcess -> {
        // code of `MyProcess`
    }

Here, `MyProcess` is a constructor (defined with `->`).
This means that whenever we use the word `MyProcess` in the program,
the value of `MyProcess` will be reevaluated,
so whenever we use the constructor `MyProcess`, we will get a fresh process,
instead of the same one every time.

*A Note About Terminology*: The processes created from the constructor `MyProcess` are called
"instances of `MyProcess`",
or simply "`MyProcess` processes".
Also, sometimes we may call `MyProcess` a "process"
instead of a constructor, although this is technically wrong.

As we will see soon, it is very useful to have multiple instances
of the same process in a program
(possibly with multiple instances running simultaneously).
This is why we almost always define processes inside of a constructor.
This is also why all the mathematical processes (like `Math::Sin`, `Sum`, etc.)
are actually constructors.

Example &ndash; Processes in Action
-----------------------------------

Let's see a simple program that uses a number of processes
to do something useful.
This program outputs all the prime numbers up to 100.

    // Gets two numbers `a`, `b`.
    // Outputs .True if `a` divides `b`, or .False otherwise.
    Divides -> {
        a, b := in
        out (b % a == 0)
    }

    // Gets a number `n`.
    // Outputs .True if `n` is prime, or .False otherwise.
    IsPrime -> {
        n := in
        // Loop for i = 2, 3, ..., n - 1
        i := 2
        while i < n {
            // Send `i` and `n` to a new instance of `Divides`,
            // receive the result.
            if [Divides i n] {
                // If `Divides` outputs .True, output .False and stop.
                out .False
                stop
            }
            i += 1
        }
        // If nothing divides `n`, output .True.
        out .True
    }

    // Outputs all the prime numbers.
    // Note that `Primes` is a variable, not a constructor.
    // This means that `Primes` always contains the same process,
    // not a fresh process every time.
    // Also note that `Primes` will not run forever,
    // despite containing an infinite loop,
    // because it pauses whenever it outputs something,
    // and `Main` will only read a finite number of its outputs.
    Primes := {
        // Loop for i = 2, 3, 4, ...
        i := 2
        loop {
            // Send `i` to a new instance of `IsPrime`,
            // and receive the output.
            if [IsPrime i] {
                // output `i` if it is prime.
                out i
            }
            i += 1
        }
    }

    // Output all the prime numbers up to 100.
    Main -> {
        loop {
            // Receive a prime number from `Primes`.
            prime := [Primes]
            // If it is over 100, terminate the program.
            if prime > 100 { stop }
            // Output the prime number.
            out prime
        }
    }

Range
-----

`Range` is a very handy builtin process,
which outputs all the numbers in a given range.
You can use `Range` in a number of ways:

`Range n`: outputs all the numbers from 0 to `n`, excluding `n`:

    Main -> {
        r := Range 3
        Assert ([r] == 0)
        Assert ([r] == 1)
        Assert ([r] == 2)
        // `r` has stopped.
    }

`Range m .To n`: outputs all the numbers from `m` to `n`, excluding `n`
(in this form, the first and third inputs to `Range` are numbers,
and the second input is the symbol `.To`).

`Range m .Through n`: outputs all the numbers from `m` to `n`, including `n`:

    Main -> {
        r := Range 11 .Through 14
        Assert ([r] == 11)
        Assert ([r] == 12)
        Assert ([r] == 13)
        Assert ([r] == 14)
        // `r` has stopped.
    }

`Range` with a `.Step` parameter: you can add two additional inputs
to any of the above forms of `Range`:
the symbol `.Step` followed by a number called the step size.
This will make `Range` go over the range in steps of the given size:

    Main -> {
        r := Range 3 .Step 3 .To 12
        Assert ([r] == 3)
        Assert ([r] == 6)
        Assert ([r] == 9)
        // `r` has stopped
    }

`Range` with `Infinity`: if you set the end of the range to `Infinity`,
you'll get an infinite range.

`Range` with a negative step: if the number you put after `.Step` is negative,
you'll get a range that goes backwards:

    Main -> {
        r := Range 5 .Step (-1) .To (-Infinity)
        n := 5
        loop {
            Assert ([r] == n)
            n -= 1
        }
    }

While Statement with Receive Expression
---------------------------------------

There is another form of `while` loop:
`while x := [process]` or `while x = [process]`.
In this kind of a `while` loop,
we repeatedly read the outputs of the given process, until we fail to read.
For every output, we assign the output to a variable,
and run the body of the loop.
For example:

    Main -> {
        proc := {
            out 1 2 3
        }
        while i := [proc] {
            out (i ^ 2)
        }
    }
    // Output:
    // 1
    // 4
    // 9

This kind of loops is often used together with a range.
So the above example could also be written:

    Main -> {
        while i := [Range 1 .Through 3] {
            out (i ^ 2)
        }
    }

(An important thing to note about this kind of loop is that
even though we poll the process in the brackets (`[process]`) for output
multiple times, we create it only once.
So in the above example, the program created only one `Range` process,
and passed it the inputs `1 .Through 3` once.
Then it polled it multiple times to get the outputs `1 2 3`.)

Talking to the User
-------------------

We already saw that inside the `Main` process, you can use `in` and `out`
to read user input and write output to the user.
But inside other processes, `in` and `out` do something different.
Sometimes it is convenient to talk the user from a process other than
`Main`, and for that, you can use the `User` process.

Whenever you send an input to `User`, `User` shows it to the user.
Whenever you try to read the output of `User`,
`User` reads a line of user input and outputs it.
So, counterintuitively,
sending input to `User` is the same as writing output in `Main` ,
and receiving the output of `User` is the same as reading input in `Main`.
For example:

    Greet -> {
        // Write to the user.
        User "What is your name?"
        // Read from the user.
        name := [User]
        // Write "Hi, <name>!" to the user.
        // `Cat` is a process that creates a string.
        User [Cat "Hi, " name "!"]
    }

    Main -> {
        Greet
    }

When we run this program, we may see the following output:

    What is your name?
    [User types: John Doe]
    Hi, John Doe!

Aggregator Processes
--------------------

Aggregators are a very useful family of processes.
To show what aggregators are, we'll start by defining a very simple process:

    Sum -> {
        a, b := in
        out (a + b)
    }

This process takes two numbers and finds their sum.
This process is very useful, but it has one big limitation:
it always takes exactly two inputs.
Let's change it, so it can take any number of inputs,
and find their sum:

    Sum -> {
        // Create a variable `sum`, to hold the current sum.
        sum := 0
        // Add each input to `sum`.
        while x := in {
            sum += x
        }
        // Output the final sum.
        out sum
    }

This is much better.
For example, to find the sum of the numbers 1, 2, 3, 4, 5,
we can now write: `[Sum 1 2 3 4 5]`.

Let's make one final improvement:

    Sum -> {
        sum := 0
        loop {
            while x := in {
                sum += x
            }
            out sum
        }
    }

All we did was add a loop around our code.
What this means is that the process doesn't stop anymore
after it outputs the sum &ndash;
instead, it goes back to reading more inputs.
This enables us to use a `Sum` process as an accumulator for numbers.
We can send it numbers to add them to the sum,
and whenever we want, we can use a *receive* expression
to read the current sum.
For example:

    Main -> {
        mySum := Sum
        // Add some numbers to the sum.
        mySum 1 2 3
        // Read the sum.
        Assert ([mySum] == 6)
        // Add some more numbers to the sum:
        mySum 4
        mySum 5
        // Read the sum again:
        Assert ([mySum] == 15)
        // Reset the sum:
        mySum (-15)
        // Read the sum, twice:
        Assert ([mySum] == 0)
        Assert ([mySum] == 0)
    }

This kind of process is called an aggregator.
An aggregator is a process that reads any number of inputs,
and stores some kind of statistics or information about all the inputs
it has read so far.
You can use a *receive* expression to get the current state of the aggregator.

In fact, there already is a builtin `Sum` process in Malakh.
Here are a few other builtin aggregators:

- `Math::Product`: finds the product of its inputs. For example:\
  `Assert ([Math::Product 2 3] == 6)`.
- `Math::Mean`: finds the mean (average) of its inputs. For example:\
  `Assert ([Math::Mean 10 8] == 9.0)`
- `Min`: finds the minimum of its inputs.
  The inputs may be either numbers or strings, but not both.
  For example:\
  `Assert ([Min 6 2 7] == 2)`.
- `Max`: finds the maximum of its inputs. Similar to `Min`.
- `Data::Count`: finds the number of inputs it has received.
  For example:\
  `Assert ([Data::Count "foo" .bar .Baz] == 3)`

String Processes
----------------

`Cat` is an aggregator that creates a string.
`Cat` takes any number of inputs,
and creates a string by concatenating them:

    Main -> {
        out [Cat "Hello, " "World!"]  // Output: Hello, World!
        out [Cat .Hello .World]       // Output: HelloWorld
        out [Cat 1 2 3]               // Output: 123
        name := Cat
        name "B" "I" "N" "G" "O"
        out [name " was his name-O"] // Output: BINGO was his name-O
    }

`String::Length` is a process that gets the length of a string
(in bytes):

    Main -> {
        Assert ([String::Length "foo"] == 3)
        Assert ([String::Length ""] == 0)
        Assert ([String::Length "supercalifragilisticexpialidocious"] == 34)
    }

`String::FromCharCodes` is an aggregator that
creates a string from character codes.
It gets any number of integers, which represent Unicode characters,
and converts them to a string:

    Main -> {
        out [String::FromCharCodes 97 98 99] // Output: abc
        out [String::FromCharCodes 0x1f414]  // Output: 🐔
    }

`String::CharCodes` is the opposite of `String::FromCharCodes`.
It outputs all the character codes in a string:

    Main -> {
        chars := String::CharCodes "abc"
        Assert ([chars] == 97)
        Assert ([chars] == 98)
        Assert ([chars] == 99)
    }

TODO: add more processes.

Simple Data Structures
----------------------

Data structure are a special family of processes,
which are used to store data.

### Stack
One of the simplest data structures is a `Stack`.
A `Stack` is a process that can store data of any kind.
When you create a stack, it starts empty, and doesn't store any values.
Whenever you send it an input, the stack stores that input internally.
The stack also remembers the order in which it got its inputs.
When you try to read from a `Stack` (using a *receive* expression),
the stack outputs the newest value stored in it,
and also removes that value from its internal storage.
When the stack is empty, it doesn't output anything:

    Main -> {
        myStack := Stack
        // Now the stack is empty.
        myStack 1 2 3
        // Now the stack contains the numbers 3, 2 and 1.
        // 3 is the newest value, 2 is the second-newest, 1 is the oldest.
        Assert ([myStack] == 3)
        // Now the stack contains only 2 and 1.
        Assert ([myStack] == 2)
        // Now the stack contains only 1.
        Assert ([myStack] == 1)
        // Now the stack is empty.
        if x := [myStack] {
            // This line will never run,
            // because we cannot read from an empty stack.
            Assert .False
        }
        myStack 4
        // Now the stack contains 4.
    }

One important use case for a `Stack` is to reverse a list of values.
For example:

    // Read some lines of input.
    // Output them back, in reverse order.
    Main -> {
        stack := Stack
        while line := in {
            stack line
        }
        while line := [stack] {
            out line
        }
    }

### Queue

Another data structure, very similar to a `Stack`, is a `Queue`.
A `Queue` can also receive any number of values and store them internally.
The difference is that a `Queue` will always output the oldest value,
instead of the newest one
(and also removes it from its storage, after outputting it):

    Main -> {
        q := Queue
        q 1 2 3
        Assert ([q] == 1)
        Assert ([q] == 2)
        q 4
        Assert ([q] == 3)
        Assert ([q] == 4)
        if x := [q] {
            Assert .False
        }
    }

### MinHeap

A `MinHeap` is a data structure that can store only numbers or strings
(but not both).
You can add new values to the `MinHeap` by sending them to it,
just like with a `Stack` or a `Queue`.
When you try to receive a value from the `MinHeap`,
it outputs the *smallest* value in its storage,
and removes it from the storage:

    Main -> {
        heap := MinHeap 6 3 10 6
        Assert ([heap] == 3)
        heap 7
        Assert ([heap] == 6)
        Assert ([heap] == 6)
        Assert ([heap] == 7)
        Assert ([heap] == 10)
        if x := [heap] {
            Assert .False
        }
    }

One important use case for a `MinHeap` is to sort a list of values:

    // Read some lines of input.
    // Output them back, sorted lexicographically.
    Main -> {
        lines := MinHeap
        while line := in {
            lines line
        }
        while line := [lines] {
            out line
        }
    }

### MaxHeap

A `MaxHeap` is the same as a `MinHeap`,
but always outputs and removes the largest value in its storage,
instead of the smallest one.
This can be useful when you want to sort a list of values
from largest to smallest.

Recursion
---------

It is possible to define a constructor whose definition refers to itself.
For example, we will now define a process that computes the Fibonacci sequence.
As a reminder, the Fibonacci sequence is defined as follows:

    f(0) = 0
    f(1) = 1
    f(n) = f(n - 1) + f(n - 2)  (where n > 1)

Here's a process that computes it:

    Fibonacci -> {
        // Read a number n.
        n := in
        if n == 0 {
            // Base case: n = 0.
            out 0
        }
        else if n == 1 {
            // Base case: n = 1.
            out 1
        }
        else {
            // When n > 1, compute f(n - 1) and f(n - 2) and output their sum.
            out ([Fibonacci (n - 1)] + [Fibonacci (n - 2)])
        }
    }

Now, let's look at a more advanced example.
We'll define our own version of `MinHeap`.
The following process is equivalent to the standard library's `MinHeap`,
albeit much slower:

    MinHeap -> {
        // Read first item.
        // The variable `smallest` will always hold
        // the smallest value in the heap.
        smallest := in
        // `rest` will hold the rest of the items.
        rest := MinHeap
        // Repeatedly add new items, or output items.
        loop {
            // Add new items to the heap.
            while item := in {
                if item < smallest {
                    // If the new item is the smallest,
                    // replace `smallest` with the new item,
                    // and add the old `smallest` to `rest`.
                    rest smallest
                    smallest = item
                }
                else {
                    // If the new item is not the smallest,
                    // add it to `rest`.
                    rest item
                }
            }
            // When there are no more items to add,
            // Output the smallest item.
            out smallest
            // Remove `smallest` from the heap.
            // Either replace it with the smallest item of `rest`,
            // or, if `rest` is empty, replace it with a new input.
            smallest = [rest]
            else {
                smallest = in
            }
        }
    }

*Note*: this definition of `MinHeap` is very odd,
and not at all what we typically write in Malakh.
But if you understand this definition,
it means you really understand the topics in this tutorial well.

Switch Statement
----------------

A `switch` statement takes a value and compares it to a number of `case`s.
When it finds a `case` that matches the value,
it executes the code associated with that case.
For example:

    // Gets the name of an animal.
    // Outputs the sound that animal makes.
    Main -> {
        out "What is your favorite animal?"
        animal := in
        switch animal {
        
        case "cat":
            out "Meow!"
        
        case "dog":
            out "Woof!"

        case "cow":
            out "Moo!"

        }
    }

If none of the cases match the value, the process crashes.
You can avoid that by providing a `default` case,
which runs when none of the other `case`s match:

    // Gets the name of an animal.
    // Outputs the sound that animal makes.
    Main -> {
        out "What is your favorite animal?"
        animal := in
        switch animal {
        
        case "cat":
            out "Meow!"
        
        case "dog":
            out "Woof!"

        case "cow":
            out "Moo!"

        // This will run when the input is not "cat", "dog", or "cow".
        default:
            out "Ummm..."

        }
    }

If you want to do nothing by default, you can leave the `default` case empty.

Unlike in some other languages, you don't have to put a `break` statement
in the end of every case.
The `switch` statement simply ends once one of its
`case`s (or `default`) ends.
In fact, `break` statements don't have any special meaning inside `switch`.

You can match multiple different values in the same `case`.
The `case` will run if any of those values matches:

    case "mouse", "rat", "squirrel":
        out "Squeak!"

Basic Object-Oriented Programming
---------------------------------

In this section, we will see how to write simple object-oriented programs
in \*Langauge Name\*.
To showcase this, we will start with the `Sum` process we created earlier,
and add *methods* to make it easier to use.
For reference, here is the definition of `Sum` from earlier:

    Sum -> {
        sum := 0
        loop {
            while x := in {
                sum += x
            }
            out sum
        }
    }

Whenever you send a number to a `Sum`, it adds it to its internal variable,
and whenever you read from a `Sum` (with a *receive* expression),
it outputs the value of the variable.
But perhaps we want to add more abilities to our `Sum`:
for example, maybe we want to be able to reset the variable,
or to multiply its value by a certain amount, or something similar.
Let's implement both of these features:

    Sum -> {
        // `sum` is initially zero.
        sum := 0
        loop {
            // Read an input.
            // If there is no input, output the sum instead.
            cmd := in
            else {
                out sum
                continue
            }
            // If input is a number, add it to the sum as usual.
            if [IsNumber cmd] {
                sum += cmd
                continue
            }
            // If input is a symbol:
            switch cmd {
            
            // If input is `.Reset`, reset the sum.
            case .Reset:
                sum = 0
            
            // If input is `.Multiply`,
            // multiply the sum by a given number.
            case .Multiply:
                sum *= in

            // Since there is no `default` case,
            // `Sum` will crash if it gets any other input.
            }
        }
    }

Our new `Sum` can do all the things it could do before,
but it also accepts two special inputs: `.Reset` and `.Multiply`.
When it gets one of those, it does something special.
For example:

    Main -> {
        sum := Sum
        sum 4 5
        Assert ([sum] == 9)
        sum 1
        Assert ([sum] == 10)
        sum .Reset
        Assert ([sum] == 0)
        sum 3 .Reset 4
        Assert ([sum] == 4)
        sum .Multiply 2
        Assert ([sum] == 8)
    }

This is what object-oriented programming is all about.
We define a process that accepts symbols as input,
and has a special behavior for each symbol.
The symbols which the process accepts are called its *methods* &ndash;
so our `Sum` has methods `.Reset` and `.Multiply`.
A process that has methods is called an *object*.

In a later section, we will expand on this subject,
and show you how to implement more advanced object-oriented patterns.

Inout Statement
---------------

The `inout` statement lets one process mimic the behavior of
another process.
`inout` looks like this:

    Process1 -> {
        Some .Code
        inout Process2
        Some .More .Code
    }

Here, when `Process1` reaches the `inout` statement, it will start mimicking
`Process2`.
Then, whenever `Process2` outputs something, `Process1` will output the
same thing.
whenever `Process2` starts waiting for input, `Process1` will also start
waiting for input, then pass the input to `Process2`.
Once `Process2` stops, `Process1` will stop mimicking it,
and go back to whatever it was doing.

For example, let's use the `inout` keyword to implement a "game library".
First, we will write a number of processes that represent text-based games.
Each such process reads input from the user and writes back to the user,
using `in` and `out` commands.
Out `Main` process will ask the user which game they'd like to play,
and then start that game:

    // First game - Chess.
    Chess -> {
        out "Starting a game of chess."
        out "Would you like to be black or white?"
        playerColor := in
        // And so on...
    }

    // Second game - Colossal Cave Adventure.
    ColossalCaveAdventure -> {
        out "YOU ARE STANDING AT THE END OF A ROAD BEFORE A SMALL BRICK BUILDING."
        // And so on...
    }

    // Third game - Hunt the Wumpus.
    HuntTheWumpus -> {
        out "I SMELL A WUMPUS"
        // And so on...
    }

    Main -> {
        loop {
            out "Which game would you like to play?"
            switch in {

            case "Chess":
                // Start mimicking `Chess`.
                inout Chess
            
            case "Colossal Cave Adventure":
                // Start mimicking `ColossalCaveAdventure`.
                inout ColossalCaveAdventure
            
            case "Hunt the Wumpus":
                // Start mimicking `HuntTheWumpus`.
                inout HuntTheWumpus
            
            default:
                out "We don't have that game. Sorry."

            }
        }
    }

In this code, we used the `in` and `out` keywords inside our games,
to communicate with the user.
To make this work, we had to use `inout` in our `Main` process,
to pass the games' output to the user, and to pass the user's input
to the games.

Forking
-------

Every process has the ability to "split" itself into many child-processes.
This is called `fork`ing, and allows us to create some highly advanced
object-oriented code.

In order to `fork` (that is, *split*) itself,
a process has to use a `fork in` expression,
When a `fork in` expression runs, the current process pauses and starts
waiting for input &ndash; just like with an `in` expression.
However, once the process gets an input, it does't resume running.
Instead, it creates a *child process* of itself, and sends the input to
the child.
The child process is a process just like the original (parent) process,
and it contains the same code and variables as its parent.
The child process runs the same code as its parent, starting immediately
after the `fork in` expression.
The result of the `fork in` expression, in the child process,
is the input that was passed to it.
Crucially, after the parent process creates the child process,
it remains in the same state as before.
it just pauses itself again and waits for more input.
If it then gets a new input, it creates a new child process, and so on.

This whole procedure allows us to create multiple copies of the same process,
that differ only in the result of the `fork in` expression in each one of them:

    TellUser -> {
        s := fork in
        User s
    }

    Main -> {
        tell := TellUser
        tell "Hello"   // Output: Hello
        tell "Goodbye" // Output: Goodbye
    }

This program outputs the word "Hello" and then "Goodbye".
Let's take a look at how this works, step by step:

1. `Main` creates a `TellUser` process.
2. The `TellUser` process starts running.
    It immediately pauses itself when it reaches the `fork in` expression,
    and waits for input.
3. `Main` sends the input "Hello" to the `TellUser` process.
4. The `TellUser` process gets the input, and creates a child process.
    It passes the input "Hello" to the child,
    and immediately goes back to waiting for input.
5. The child starts running.
    It receives the input "Hello" and stores it in the variable `s`.
6. The child executes the statement `User s`.
    It outputs "Hello" to the user, and stops.
7. `Main` resumes running.
  It now sends the input "Goodbye" to the parent `TellUser` process.
8. Again, the parent creates a child and passes it the input "Goodbye".
    The parent immediately goes back to waiting for input.
9. The new child starts running.
    It receives the input "Goodbye" and stores it in the variable `s`.
10. The new child executes the statement `User s`.
    It outputs "Goodbye" to the user, and stops.
11. `Main` resumes running, but it has reached the end of its code.
    `Main` stops and the program terminates.

Send Expressions with Forking
-----------------------------

We have seen earlier that a *send* expression always yields the process
that it got. For example:

    Main -> {
        foo := { in }
        Assert (foo == (foo .Spam))
    }

With forking, this is no longer true.
When you send input to a process, and the process forks itself,
the *send* expression yields the new child process, not the parent.
This means that after creating a child (by sending an input to its parent),
you can immediately send the child more inputs, or read its output:

    Main -> {
        add := {
            first := fork in
            second := in
            out (first + second)
        }
        out [add 1 2] // Output: 3
        out [add 3 4] // Output: 7
    }

In the above code, we are sending the numbers 2 and 4 directly to the child
processes, and receiving the outputs directly from the children.

Shared Variables
----------------

When a process forks itself, it also shares all of its variables with its
child processes.
This means that every variable that existed when the process forked itself
is also accessible to its children.
If a child modifies one of those variables, all the other children
can see the change too.

On the other hand, when a child process creates a variable
after the `fork in`, it creates it only for itself.
Another child process cannot access that variable
(though it may create another variable with the same name).
These "private" variables includes all the variables that are 
created on the same line as the `fork in`, or the following lines.
For example:

    Foo -> {
        shared := 0
        fork in
        unshared := 0
        shared += 1
        unshared += 1
        User [Cat "shared = " shared ", unshared = " unshared]
    }

    Main -> {
        foo := Foo
        foo .Spam // Output: shared = 1, unshared = 1
        foo .Ham  // Output: shared = 2, unshared = 1
        foo .Eggs // Output: shared = 3, unshared = 1
    }

In the above program, there was only one variable named `shared`,
but three separate variables named `unshared`.

This Keyword
------------

Every process contains a special variable called `this`.
`this` refers to the current process:

    Main -> {
        myProc := { out this }
        Assert ([myProc] == myProc)
    }

In a child process, `this` always refers to the parent process,
not to the child.
To access the current child of the process,
use the special value `fork this` instead:

    Main -> {
        myProc := {
            fork in
            out this
            out fork this
        }
        child := myProc .Spam
        Assert ([child] == myProc)
        Assert ([child] == child)
    }

Reentrant Objects
-----------------

Let's go back to object-oriented programming.

When defining an object in Malakh, it is often convenient to define
one of its methods in terms of another method.
For example, consider this process (similar to one we've already seen):

    Sum -> {
        sum := 0
        loop {
            switch in {
            
            case .Add:
                sum += in
            
            case .Subtract:
                sum -= in

            }
        }
    }

when implementing the `.Subtract` method, we may be tempted to define it like
this instead:

    // Subtracting is the same as adding a negative number.
    case .Subtract:
        this .Add (-in)

Doing so would be especially tempting if our `.Add` and `.Subtract` were
longer and contained more code,
and this could save us a lot of typing.

Unfortunately, this code doesn't actually work.
The line `this .Add (-in)` wouldn't work, because a process can never send
input to itself
(and in general, you can't send input to a process that's already running).
To solve this, we have to modify our `Sum` process,
so it forks itself after reading input:

    Sum -> {
        sum := 0
        switch fork in {
        
        case .Add:
            sum += in
        
        case .Subtract:
            this .Add (-in)

        }
    }

Now, the line `this .Add (-in)` works as expected,
because whenever it runs, the process that executes it is a child process.
The statement `this .Add (-in)` creates another child process
and sends the inputs: `.Add (-in)` to it.
So no process actually sends input to itself.

Finally two important points about this code:

1. The `this` keyword in the code refers to the parent process,
so sending input to `this` causes the parent to create
a new child process.
2. The variable `sum` is created before the `fork in` command,
so it is shared among all child processes.
When one child modifies it, it affects all the other children.

An unfortunate consequence of this change to the code is that we cannot send
multiple inputs to our `Sum` process on one line, anymore.
For example, with with our original `Sum` process, we could write:

    s := Sum
    s .Add 1 .Add 2 .Subtract 3

now our `Sum` can accept only one method at a time, so we must write:

    s := Sum
    s .Add 1
    s .Add 2
    s .Subtract 3

Lists
-----

`List` is another data structure process, similar to `Stack` and
`Queue`, but much more advanced.
It is also an object, implemented with forking.

To create a list, use the constructor `List`.
Like a stack or a queue, a list contains a sequence of values.
The list starts empty, and you can use the `.Push` method to add items to it.

You can use the `.Length` method to get the list's length:

    Main -> {
        myList := List // create an empty list
        Assert ([myList .Length] == 0)
        myList .Push 1 2 3 // Add the numbers 1, 2, 3 to the list.
        Assert ([myList .Length] == 3)
        myList .Push .Foo .Bar // Add the symbols .Foo, .Bar to the list.
        Assert ([myList .Length] == 5)
    }

Each item in the list has an index, from `0` to `n-1`
(where `n` is the length of the list).
You can read any item from the list by its index:

    Main -> {
        list := List
        list .Push "foo" "bar"
        list .Push "baz"
        out [list 0] // Output: foo
        out [list 1] // Output: bar
        out [list 2] // Output: baz
    }

If the index is not in the range `0`, ..., `n-1`, the list outputs nothing.

You can replace an existing item in the list using `list i .Set x`:

    Main -> {
        list := List
        list .Push 1 2 3
        Assert ([list 2] == 3)
        list 2 .Set 10 // Replace the third item with 10
        Assert ([list 2] == 10)
    }

To remove an item from the list, use `list i .Remove`:

    Main -> {
        list := List
        list .Push 10 .Bad 20 // list will contain 10, .Bad, 20
        list 1 .Remove // Remove the .Bad
        Assert ([list 0] == 10)
        Assert ([list 1] == 20)
    }

The `.Remove` method also outputs the value it removed.

It is very common to remove items from the end of the list.
To do that, you can use the `.Pop` method:

    Main -> {
        list := List
        list .Push 1 2 3
        Assert ([list .Pop] == 3)
        // Instead of `list .Pop`, you could write: `list 2 .Remove`.
        Assert ([list .Length] == 2)
    }

To add a new item in the beginning or the middle of the list,
use `list i .Insert x`.
This will add a new item `x` to the list, just before the `i`th item:

    Main -> {
        list := List
        list .Push "a" "d" "f"
        list 1 .Insert "c"
        list 1 .Insert "b"
        list 4 .Insert "e"
        Assert (
            [list 0] == "a" and [list 1] == "b" and [list 2] == "c" and
            [list 3] == "d" and [list 4] == "e" and [list 5] == "f"
        )
    }

The `.Each` method outputs all the items in the list:

    Main -> {
        list := List
        list .Push "foo" "bar" "baz"
        while s := [list .Each] {
            out s
        }
    }
    // Output:
    // foo
    // bar
    // baz

`List::Of` is a process that helps you create lists more easily.
`List::Of` gets any number of values, and outputs a list that contains them:

    Main -> {
        list := [List::Of .A .B .C]
        Assert ([list 0] == .A and [list 1] == .B and [list 2] == .C)
    }

Process State
-------------

The `Process::State` process takes another process,
and tells us that process's state, i.e. what that process is doing
right now.

The output of `Process::State` will be one of the following:

- `.Run`: the process is running.
- `.Stop`: the process has finished running.
- `.Out`: the process is executing an `out` statement,
    and is waiting for another process to read its output.
    It will resume running once someone reads it.
- `.In`: the process is executing an `in` expression,
    and is waiting for input.
    It will resume running once it gets some input.
- `.OptIn`: the process is executing an optional `in` expression,
    and is waiting for input.
    It will resume running once it gets some input &ndash;
    but it can also resume without input, if requested.
- `.ForkIn`: the process is executing a `fork in` expression,
    and is waiting for input.
    When it gets input, it will create a child process,
    but stay in the `.ForkIn` state.
- `.Err`: the process has encountered an error.

For example:

    Main -> {
        Assert ([Process::State this] == .Run)
        Assert ([Process::State {}] == .Stop)
        Assert ([Process::State (Math::Sin 1.0)] == .Out)
        Assert ([Process::State Math::Sin] == .In)
        Assert ([Process::State Sum] == .OptIn)
        Assert ([Process::State List] == .ForkIn)
        Assert ([Process::State {1 / 0}] == .Err)
    }

Control Flow
------------

In this section, we will define the formal semantics of processes,
and in particular, what causes a process to run, stop, pause, and so on.
Usually you don't need to think about these rules, as the behavior of
processes tends to be very intuitive, but there are some edge-cases where this
is important.

As you already saw, at any given moment, each process can be in one of seven
states: `.Run`, `.Stop`, `.Out`, `.In`, `.OptIn`, `.ForkIn`, `.Err`.
The program has a special stack data structure called
the *process stack*, which holds all the processes that are in the
`.Run` state.
At any given moment, the only process that is actively running is the one on
top of the process stack.
(There may be multiple processes in the `.Run` state at the same time,
but the interpreter is only able to run one of them at any given moment,
and it always chooses to run the one at the top of the stack.)

Whenever you create a process, it always starts in the `.Run` state,
and automatically gets placed on top of the stack.
This means that each process that you create starts running immediately.
When the program starts running, the first process that gets created is
`Main`, and it too starts in the `.Run` state, and gets placed on the stack.

The stack may only contain processes that are in the `.Run` state.
If the state of the top process of the stack changes from `.Run` to something
else, it immediately gets removed from the stack.
(Processes on the stack, other than the one on top, will never change their
state.)

The following rules describe the transitions between states:

- A process in the `.Run` state may change its state only while it is actively
  running (i.e. it is on top of the stack).
  It may change its state depending on the kind of statements it executes:

  - If it executes an `out` statement, its state will change to `.Out`.
  - If it executes an `in` expression, its state will change to `.In`,
    or `.OptIn` if it is an optional input.
  - If it executes a `fork in` expression, its state will change to `.ForkIn`.
  - If an error occurs, its state will change to `.Err`.
  - If it reaches the end of its code, or executes a `stop` statement,
    its state will change to `.Stop`.
  
  After the process changes its state, it gets removed from the stack,
  which means that the process below it on the stack starts running.
- A process in the `.Stop` state will remain in that state forever.
- A process in the `.Out` state has a value which it is trying to output.
  It has an *output slot*, which holds that value.
  When another process receives that output (using a *receive* expression),
  it reads the value of the output slot, then empties it.
  By doing so, it notifies the process with the output slot that its output
  has been received, and that process changes its state from `.Out` to `.Run`,
  and gets pushed to the top of the stack.
- A process in the `.In` state waits for another process to send it input.
  When another process sends it input (using a *send* expression),
  its state changes from `.In` to `.Run`, and it gets pushed to the top of
  the stack.
- A process in the `.OptIn` state waits for another process to send it input,
  like in the `.In` state.
  But in addition, another process may send it a special signal that tells it
  there is no input.
  When the process either gets input, or gets the "no input" signal,
  its state changes to `.Run` and it gets pushed to the top of the stack.
- A process in the `.ForkIn` state will remain in that state forever.
  If another process sends it input, it creates a new child process,
  and gives the input to it.
  The child process will start in the `.Run` state, and get pushed
  to the top of the stack. \
  TODO: maybe a `finally` statement may cause a parent to change its state.
- The `.Err` state is very similar to `.Out`:
  each process has an *error slot*, where it holds errors.
  Whenever a process encountered an error, its state changes to `.Err`,
  and information about the error gets stored in the error slot.
  The process will remain in the `.Err` state until another process receives
  the error, at which point it resumes running.\
  We will discuss the `.Err` state in more detail in the section about
  error handling.

**Send Expressions:** when a process `p1` executes a *send* expression:
`p2 x`, the process `p2` must be in one of the state `.In`, `.OptIn`,
`.ForkIn` &ndash; otherwise, `p1` crashes.
The process `p2` gets the input, and acts as described above.

**Receive Expressions:** when a process `p1` executes a *receive* expression:
`[p2]`, the process `p2` must be in the `.Out` state or in the `.OptIn` state
&ndash; otherwise, `p1` crashes. \
If `p2` is in the `.Out` state, `p1` reads its output slot and empties it,
as described above. \
If `p2` is in the `OptIn` state, then `p1` tries to make it switch its state
to `.Out`. To do so, it sends `p2` the "no input" signal (as described above).
This causes `p2` to change its state to `.Run` and run for a while.
Once `p2` gets removed from the stack, `p1` resumes running.
`p1` checks if the state of `p2` is now `.Out`. If not, `p1` crashes.
If the state of `p2` is `.Out`, `p1` reads its output as usual.\
In particular, if the state of `p2` was `.OptIn`, and remains `.OptIn` after
one "no input" signal, `p1` won't try to send it another "no input" signal.

**Optional Receive Expressions:** an optional receive expression `[p2]`
acts just like a regular receive expression, except that whenever the process
that executes it would crash because `p2` is in the wrong state,
it doesn't crash, but runs some alternative code, as defined in the program.

Err Statement
-------------

Until now, in all the programs we've written, we assumed that everything goes
according to plan:
processes always get the input they were expecting,
division by zero never occurs, assertions always succeed, and so on.
But Malakh has ways of dealing with these situations too, using the
`err`, `throw` and `try` statements.

The simplest of these tools is the `err` statement.
An `err` statement looks very much like an `out` statement, for example:

    err .IoError "file foo.txt not found"
    err 1 2 3 4 5

When a process executes an `err` statement, it takes the sequence of values
after `err`, and places them together in its *error slot*
(each process has an error slot, analogous to the output slot, for this
purpose). Then its state changes to `.Err`, and it waits until
another process receives the error.

An `err` statement is almost identical to `out`, with two main differences:
- An `err` statement outputs a sequence of values together, and not one at a
  time. For example, this statement:
    
      err .Foo .Bar
    
  outputs the sequence `.Foo .Bar`, and is different from the following
  statements:

      err .Foo
      err .Bar

  which output two separate errors.

- An `err` statement outputs an error, not a regular output.
  You can't receive an error with a *receive* expression.

The values after `err` are called *error values*.
Usually, error values are either:

- Strings describing the error, called *error messages*.
- Symbols describing the general category of the error, called *error tags*.

For example, this error: `err .IoError "file foo.txt not found"`
has one error message and one tag.

If an `err` statement appears in `Main`, the error values get output to the
user (like with `out`), but in a different format, marking it as an error.

Throw Statement
---------------

When an error occurs, you'll usually want to stop the process
immediately after raising the error, for example:

    Divide -> {
        a, b := in
        if b == 0 {
            err .MathError "division by zero"
            // No sense in continuing if b = 0.
            stop
        }
        out (a / b)
    }

This pattern is in fact so common that there's special syntax for it:

    Divide -> {
        a, b := in
        if b == 0 {
            throw .MathError "division by zero"
        }
        out (a / b)
    }

As you can see, a `throw` statement is exactly like `err`, except that it
also stops the process.

Error Propagation
-----------------

When a process sees another process raise an error, it may throw the same
error too, "out of sympathy".
This is called *error propagation*, and it ensures that execution won't
continue if a dangerous error has occured.

Specifically, if process `p` performs some operation on process `q` , whose
state is `.Err`, `p` may enter the `.Err` state too, with the same error as
`q`. After another process receives `p`'s error, `p` will stop.

Operations that cause error propagation are:

- An attempt to receive from process `q`.
- An attempt to send input to process `q`.
- A statement that contains only the name of a variable or a constructor
  and nothing else. For example, in the following program:

      Foo -> {
        Bar
      }
    
  `Foo` will throw an error if `Bar` is a process in the `.Err` state.
- A statement that contains only a send expression. For example, in the
  following program:

      Foo -> {
          Bar Baz
      }

  `Foo` will throw an error if `Bar` was in the `.Err` state before the send,
  or if `Bar` enters the `.Err` state after getting its input.
  On the other hand, it will *not* throw an error if `Baz` is a process in the
  `.Err` state.
- Any of the above, enclosed in parentheses.

**Example:**

    Thrower -> {
        throw "oh no"
    }

    Thrower2 -> {
        // Throw "oh no".
        Thrower
    }

    DelayedThrower -> {
        in
        // Throw "oh no" after getting input.
        Thrower2
    }

    Thrower3 -> {
        // This does nothing:
        DelayedThrower
        // This does nothing:
        _example1 := Thrower
        // This does nothing:
        _example2 := DelayedThrower .Spam
        // But this throws an error:
        DelayedThrower .Spam
    }

    Thrower4 -> {
        // Receiving from a process in the `.Err` state causes an error,
        // even if the receive expression is optional.
        x := [Thrower]
        else {}
    }

**Some notes:**

- Suppose that `p` propagates `q`'s error.
  Even if originally, `q` ran an `err` statement, so it's able to recover from
  the error, `p` will always *throw* the error, so it can't recover.
- `p` does not *receive* `q`'s error, it only peeks into its *error slot*
  without emptying it. So `q` stays in the `.Err` state.

Try Statement
-------------

A `try` statement is a way to "catch" errors, and continue running
after an error was thrown.

A `try` statement looks very much like a `switch` statement:

    Main -> {
        try {
            DangerousOperation
            out "operation successful"
        
        case .ErrorTag1:
            out "operation failed due to reason 1"
        
        case .ErrorTag2, .ErrorTag3:
            out "operation failed due to reason 2 or 3"
        
        default:
            out "operation failed for unknown reason"

        }
    }

A `try` statement starts by executing the first few statements, up to the
first `case` or `default`.
If these statement run without errors, the `try` statement finishes.

If these statements cause an error to be thrown (either explicitly, using
`throw`, or implicitly, for example due to error propagation or division by
zero), the error doesn't actually get thrown.
Instead, the `try` statement looks for a `case` that matches the error.

Each `case` contains a list of values (which are usually symbols).
If *at least one* of these values is equal to *at least one* of the error
values in the caught error, we say that the `case` matches the error.
In this case, the body of the `case` gets executed, then the `try` statement
finishes.

If none of the `case` statements match the error, one of two thing happens:

- If the `try` statement contains a `default` clause, the `default` clause
  runs, then the `try` statement finishes.
- Otherwise, since none of the cases has matched the error, the error gets
  re-thrown.

**Example:**

    // A "safe" division process.
    Divide -> {
        a, b := in
        if not [IsNumber a] or not [IsNumber b] {
            throw .TypeError "Divide got wrong type"
        }
        if b == 0 {
            throw .DivisionByZero
        }
        out (a / b)
    }

    Main -> {
        a, b := in
        try {
            // Divide will throw .TypeError "Divide got wrong type",
            // since `a` and `b` are strings.
            out [Divide a b]
        
        // This will not match.
        case .DivisionByZero:
            out "Oh no!"
        
        // Since there is no `default`, `Main` will rethrow the same error.

        }
    }

Try-Finally Statement
---------------------

TODO

Scope
-----

Variable and constructors in Malakh use lexical scoping.
This means that every variable that is defined inside a pair of braces,
is accessible anywhere within that pair of braces, but not outside of it.
Global variables, which are not declared inside braces, can be accessed
from anywhere inside their module. For example:

    // `Foo` can be used inside both `Bar` and `Main`.
    Foo := 123

    Bar -> {
        // `x` exists only inside `Bar`.
        x := Foo
        out x
    }

    Main -> {
        // This is a different `x` from the one in `Bar`.
        x := [Bar] * [Bar]
        proc := {
            // `proc` can access `x`.
            out (x + x)
        }
        if [proc] > 0 {
            // `y` exists only inside the `if` statement.
            y := x
            out y
        }
    }

The only exception to this rule is that each `case` or `default`
in a `switch` statement has its own scope,
as if the content of each `case` or `default` were wrapped in braces.

When defining a variable or a constructor, its name must be unique in its
scope. You cannot define a variable or a constructor with a name that's
already in use.

Because of the scoping rules, it is sometimes convenient to declare a variable
in one scope, but assign its value in another, smaller scope.
For that you can use the colon (`:`) operator, which declares a variable
without explicitly assigning it a value:

    Main -> {
        // Declare `n`.
        n:
        // Assign `n` one value or another, depending on some condition.
        if s := in {
            n = [ToNumber s]
        }
        else {
            n = 0
        }
        out n
    }

You can also declare and assign multiple variables together:

    // Gets two numbers.
    // Outputs their minimum and their maximum.
    MinMax -> {
        x, y := in
        // Declare `min` and `max`.
        min, max:
        if x <= y {
            // Assign min=x and max=y.
            min, max = x, y
        }
        else {
            // Assign min=y and max=x
            min, max = y, x
        }
        out min max
    }

The initial value of a variable created with `:`
will be the symbol `.Undefined`.

Advanced Variable Declaration and Assignment Syntax
---------------------------------------------------

You can declare multiple variables on the same line:

    // Same as: `x := 1; y := 2`
    x, y := 1, 2

You can also assign to multiple variables on the same line:

    // Same as: `x = 1; y = 2`
    x, y = 1, 2

You can mix declarations and assignments on the same line.
To do that, use the `:=` operator, but wrap each variable that already exists in parentheses:

    // Create `x` and `z`, and assign values to `x` and `z`.
    x, z := 1, 2
    // Create `y` and `w`, and assign values to `x`, `y`, `z` and `w`.
    (x), y, (z), w := .Foo, .Bar, .Baz, .Quux

To the right of the `:=` or `=` operators, you can put an `in` or *receive*
expression.
This will make the process read multiple values and assign each of them to
a different variables:

    // Read three inputs, and assign them to three variables.
    x, y, z := in

    // Same as: `a, b, c, d := 0, 1, 2, 3`.
    a, b, c, d := [Range 10]
    // The remaining outputs of the `Range` will not be read.

An underscore (`_`) is a placeholder for a variable.
If you assign any value to `_`,
that value will be discarded and not stored anywhere:

    // Read three inputs, but discard the second one.
    first, _, third := in

Implicit Loops
--------------

`...` is an operator that can replace simple `while` loops.

A very common pattern in Malakh is to take all the outputs of one
process, and pass them as inputs to another process.
For example, consider this process:

    // Converts a queue to a stack.
    QueueToStack -> {
        queue := in
        stack := Stack
        while x := [queue] {
            stack x
        }
        out stack
    }

The `while` loop reads all the outputs of `queue` and passes them to `stack`.
The `...` operator can do the same thing:

    // Converts a queue to a stack.
    QueueToStack -> {
        queue := in
        stack := Stack
        // Repeatedly pass the outputs of queue to stack.
        stack [queue]...
        out stack
    }

Behind the scenes, `...` works exactly like the `while` loop above,
but it is shorter to write.
If you wanted, you could shorten this process even further:

    // Converts a queue to a stack.
    QueueToStack -> {
        queue := in
        // `...` can appear in a send expression, under the usual rules.
        out (Stack [queue]...)
    }

`...` can also appear after `in` (like this: `in...`).
In this case, it repeatedly reads inputs, and passes them to a process
(stopping when there is no more input):

    // Gets any number of inputs.
    // Stores them in a list, and outputs the list.
    ListOf -> {
        list := List
        list .Push in...
        out list
    }

`...` can also appear in an `out` statement.
In this case, it repeatedly reads either inputs,
or outputs of another process, and outputs them:

    // Gets a list.
    // Outputs its elements, sorted.
    SortList -> {
        list := in
        heap := MinHeap [list .Each]...
        out [heap]...
    }

The statement `out in...` makes the process read inputs,
and immediately output each input it reads.
This is not very useful, but it works.

`...` can also appear after `err` or `throw`, with a similar meaning.

**In summary:** `...` can appear either inside a *send* expression or inside
an `out`, `err` or `throw` statement.
It follows either a *receive* expression or an `in` expression.
In all cases, the `...` operator repeatedly reads values, as many as
possible, and sends or outputs each one.

Bare Block
----------

A bare block is another type of statement.
It consists of a sequence of statements surrounded by braces (`{}`).
For example:

    Main -> {
        out 1
        {
            out 2
            out 3
        }
        out 4
    }
    // Output:
    // 1
    // 2
    // 3
    // 4

The instructions inside the bare block will run exactly once,
as if they were not wrapped in braces at all.
The only place where a bare block makes a difference is if you define
variables inside it.
Variables created inside a bare block (or any other type of block)
will be delete at the end of the block;
this is sometimes useful to conserve memory.

In particular, when defining an object, you often have some code that
initializes the object immediately after it's created,
before it starts reading input.
Variables created in this initialization code are often irrelevant to the rest
of the object's code, so it is good practice to write an "initialization
block" that handles all the initialization,
to prevent you from accidentally using those temporary variables in the
remainder of the code.

Debug Statement
---------------

TODO

Modules
-------

TODO

Design Patterns
===============

Inheritance
-----------

TODO: introduction to object-oriented programming and inheritance.

A very common pattern in object-oriented programming is to take an existing
type of object, and extend it with more data and methods.
For example, take a look at this process (similar to ones we've seen before):

    Sum -> {
        sum := 0
        switch fork in {

        case .Get:
            out sum
        
        case .Add:
            sum += in

        }
    }

Now, we may want to extend `Sum` with more features.
For example, maybe we want to add a `.Reset` method that sets the sum to
zero, or an `.Subtract` method that subtracts from the sum.
But suppose that we don't have access to the original code of `Sum` &ndash;
so instead, we'll create another type of process that wraps `Sum`,
and adds the new method to it:

    BetterSum -> {
        super := Sum
        cmd := fork in
        switch cmd {

        case .Reset:
            super .Add (-[super .Get])
        
        case .Subtract:
            super .Add (-in)
        
        default:
            inout (super cmd)

        }
    }

Our `BetterSum` wraps a `Sum` process (in a variable called `super`).
Whenever `BetterSum` gets one of the methods `.Reset` or `.Subtract`,
it runs some custom code that handles those methods.
But when it gets one of the methods `.Get` or `.Add`,
it just sends them over to `super` &ndash; `super` is a `Sum` process,
so it knows how to handle those methods.
In these cases, `BetterSum` uses an `inout` statement to mimic the behavior
of `Sum`, and read input or write output as appropriate.

This pattern of extending an object with additional methods
is called inheritance.
For example, in the above code, `BetterSum` inherits `Sum` and extends it.
`Sum` is also called a super-object of `BetterSum`, and `BetterSum` is called
a sub-object of `Sum`.

In this case, `BetterSum` only contained one variable, `super`, that contained
its super-object, and didn't contain any other data.
In more advanced cases, we will also store additional data inside the
sub-object. But the principle remains the same.

Abstract Objects
----------------

In the previous example, we had a sub-object,
which extended its super-object and used its methods.
In many cases, we will also want to do the opposite &ndash;
to use the sub-object's methods from inside the super-object.
This is also possible in Malakh,
though it will require us to modify the super-object a bit,
to make it aware that it has a sub-object.

TODO: explain more.

For example:

    // `Animal` is an abstract type.
    // It implements the methods `.Name` and `.Talk`,
    // and expects its sub-objects to implement a method called `.Sound`.
    Animal -> {
        // sub is the sub-object.
        // name is the name of the animal.
        sub, name := in
        switch fork in {
        
        case .Name:
            out name

        case .Talk:
            User [Cat name " says " [sub .Sound]]

        }
    }

    // `Dog` inherits `Animal`, and adds the methods `.Color` and `.Sound`.
    Dog -> {
        // super is the super-object.
        // color is the dog's color.
        super := Animal this in
        color := in
        cmd := fork in
        switch cmd {
        
        case .Color:
            out color

        case .Sound:
            out "woof"

        // Pass other methods to super.
        default:
            inout (super cmd)

        }
    }

    Main -> {
        dog := Dog "Laika" "white"
        out [Cat [dog .Name] " is " [dog .Color]]
        dog .Talk
    }

    // Output:
    // Laika is white
    // Laika says woof

The `[{}]` Trick
----------------

There's is a trick that's useful when defining global variables.
When you define a global variable, you must assign it a value immediately
&ndash; you cannot use multiple lines to initialize a global variable.
For example, suppose that you want to create a global list with 100 zeroes.
You cannot do this:

    GlobalList := List
    while _ := [Range 100] {
        GlobalList .Push 0
    }

(because `while` loops are not allowed outside of a process.)

To bypass this restriction, you can define a new process
that creates the initial value of the global variable:

    initGlobalList -> {
        list := List
        while _ := [Range 100] {
            list .Push 0
        }
        out list
    }
    GlobalList := [initGlobalList]

And this can be written more consicely like this:

    GlobalList := [{
        list := List
        while _ := [Range 100] {
            list .Push 0
        }
        out list
    }]
