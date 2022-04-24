# AIM
Text based music and sound design tool.

## Why
Make music in terminal without boss knowing.

## How
- Use vim
    - Or Notepad.exe, whatever makes your boat fly
	- A text editor is all you need

## Status
Not much yet. 

## What
- Node based
- Automatic (profiling?) load balancing on CPU cores
- High performance
	- Use Rust
	- Maybe look at using LLVM JIT for further speed ups
- "Not invented here" syndrome
	- To get better at math
	- Understand how DSP works

## Examples

### Simple module
You start writing a new module from scratch:

```
sine
out
```

Then you run the synth that is in the same path as the music project:

```sh
$ aim
```

Synth automatically completes the file and returns:

```
sine id0
	frequency 440
	outlets
		out
out id1
	inlets
		in
```

Then you connect "sine" node to "out" node by altering line 4:

```
…
    out id1:in
…
```

Run the synth again:
```sh
$ aim
```

And end up with this result that would play a 440Hz sine wave to the speakers:

```
sine id0
	frequency 440
	outlets
		out in:id1
out id1
	inlets
		in out:id0
```

Play the project and listen to the sine wave by adding a new line somewhere (doesn't matter which line):
```
…
<play
…
```

The synth plays the sine wave sound and removes the `<play` line you wrote. This is how you send commands to the synth, inside the text files.

## Technical rules
- All modules (e.g main.txt) has their own ID namespace
	- A node in one module can not reference another node in another module with just ID
- Indentation in modules are only tabs!
