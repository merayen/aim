# Data
- A
	- B
	- C
		- D
	- E

# Processing order

## `on_init` 
Not important.


## `on_prepare` 
Create voices in this phase.

- A
- B
- C
- E
- D

## `on_process` 
- A
- B
- C
- D
- (suspend thread, wait for D)
- E
