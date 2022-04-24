# AIM
Text based music and sound design tool.

## Why
Make music in terminal without boss knowing.

## How
- Use vim
    - Or Notepad.exe, whatever makes your boat fly
	- A text editor is all you need

## Status
Not much yet. It can parse files and output errors into them.

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
Create a new project:

```
$ mkdir my_song
$ cd my_song
$ vi main.txt
```

You start writing a new module from scratch, adding two nodes:

```
sine
out
```

Then you run the synth that is in the same path as the music project:

```
$ aim
```

Synth automatically changes the file so it looks like this:

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
```
$ aim
```

And the file gets changed again:

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

and run synth again:

```
$ aim
Press CTRL-C to stop playback
```

The synth plays the sine wave sound and removes the `<play` line you wrote. This is how you send commands to the synth, inside the text files.

### Errors
Errors are shown as hints in the stdout from the `aim` command and written into the module files themselves. Example error from a bad module file:

```
$ cat main.txt
sine
  frequency 440
	non-existing property

$ aim
main.txt:3: ERROR: Unknown property

$ cat main.txt
sine
  frequency 440
	non-existing property  # ERROR: Unknown property
```

The errors from `aim` can be retrieved directly into vim's quickfix list by running: `:cex system("aim")` inside vim.

## Technical rules
- All modules (e.g main.txt) has their own ID namespace
	- A node in one module can not reference another node in another module with just ID
- Indentation in modules are only tabs!
