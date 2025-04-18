"""Experimenting using Python itself for writing music"""
from collections import OrderedDict
from dataclasses import dataclass, field, Field
from functools import cached_property
from typing import Any, Optional

import contextvars

from aim import importing


def node(cls: type):
	assert issubclass(cls, Node)
	return dataclass()(cls)


class RestrictedPythonError(Exception):
	pass


class Entity:
	@cached_property
	def _variable(self) -> str:
		return create_variable()


class Node(Entity):
	# Placeholders
	_inlets = None
	_outlets = None

	def __add__(self, other) -> "add":
		return add(self, other)

	def __sub__(self, other) -> "sub":
		return sub(self, other)

	def __mul__(self, other) -> "mul":
		return mul(self, other)

	def __truediv__(self, other) -> "div":
		return div(self, other)
	
	def __gt__(self, other) -> "gt":
		return gt(self, other)

	def __lt__(self, other) -> "lt":
		return lt(self, other)

	def _first_outlet(self) -> Optional["Outlet"]:
		# Node is sent as input, get the first outlet
		outlets = []
		for a in dir(self):
			if not a.startswith("_"):
				b = getattr(self, a)
				if isinstance(b, Outlet):
					outlets.append(b)

		if not outlets:
			return None

		return sorted(outlets, key=lambda a:a._index)[0]

	def __post_init__(self) -> None:
		# Insert inlets and outlets into self._inlets and self._outlets
		outlets = OrderedDict()
		inlets = OrderedDict()

		# Collect our own outlets
		for k in set(dir(self)) - set(self.__dataclass_fields__.keys()):
			value = getattr(self, k)
			if isinstance(value, Outlet):
				assert value.node is None

				# Replace Outlet with a new, initialized instance for us only
				value = value.initialize(self)
				setattr(self, k, value)
				outlets[k] = value

		# Collect inlets
		for k in self.__dataclass_fields__.keys():
			if k.startswith("_"):
				continue

			value = getattr(self, k)

			if isinstance(value, Node):
				# Maybe this should have been a silent error
				# This can happen if e.g doing "out(out())", where "out" has no outlets
				assert value is not self
				assert value._outlets, f"Node {value.__class__} does not have any outlets"

				# Replace the remote node with its "default" outlet
				setattr(self, k, list(value._outlets.values())[0])
				inlets[k] = list(value._outlets.values())[0]

			elif isinstance(value, Field):
				inlets[k] = None

			elif isinstance(value, Outlet):
				if value.node:
					# Another node being connected to out input
					assert value.node is not self
					inlets[k] = value

			else:
				# E.g a constant
				inlets[k] = value

			value = getattr(self, k)

		self._inlets = inlets
		self._outlets = OrderedDict(sorted(outlets.items(), key=lambda a: a[1]._index))


class OutNode(Node):
	def __post_init__(self) -> None:
		super().__post_init__()
		_PARSE_CONTEXT.get().out_nodes.append(self)


@dataclass
class Context:
	out_nodes: list[OutNode] = field(default_factory=lambda: [])
	unnamed_counter: int = 0


@dataclass(frozen=True)
class CompilationContext:
	context: Context
	graph: dict[int, set[int]]
	node_ids: dict[int, Node]
	order: list[int]


_PARSE_CONTEXT = contextvars.ContextVar("_PARSE_CONTEXT")


class DataType:
	SIGNAL = 1
	MIDI = 3
	NONE = 4  # When not decided statically


@dataclass
class Outlet(Entity):
	datatype: DataType
	node: Optional[Node] = None  # Set later automatically

	_index_counter = 0

	def __get_outlet(self, v) -> "Outlet":
		"""Pick the first output of the node

		This is a shortcut so that the user doesn't need to always type the outlet name."""
		if isinstance(v, Node):
			return v._first_outlet()
		return v

	def __add__(self, other) -> "add":
		return add(self, self.__get_outlet(other))

	def __sub__(self, other) -> "sub":
		return sub(self, self.__get_outlet(other))

	def __mul__(self, other) -> "mul":
		return mul(self, self.__get_outlet(other))

	def __truediv__(self, other) -> "div":
		return div(self, self.__get_outlet(other))
	
	def __gt__(self, other) -> "gt":
		return gt(self, self.__get_outlet(other))

	def __lt__(self, other) -> "lt":
		return lt(self, self.__get_outlet(other))

	def __post_init__(self) -> None:
		self._index = Outlet._index_counter
		Outlet._index_counter += 1

	def initialize(self, node: Node) -> "Outlet":
		outlet = Outlet(datatype=self.datatype, node=node)
		outlet._index = self._index  # Overwrite index with our one
		return outlet


@node
class print(OutNode):
	input: Any = None


@node
class put(OutNode):
	label: Any = None
	input: Any = None


@node
class get(Node):
	label: Any = None
	output = Outlet(DataType.SIGNAL)  # The format changes dynamically


@node
class midi(Node):
	"""Read midi from connected device"""
	device_name: Any = None

	midi = Outlet(DataType.MIDI)


@node
class midifile(Node):
	"""Read midi from file"""


@node
class add(Node):
	in0: Any = None
	in1: Any = None

	output = Outlet(DataType.SIGNAL)


@node
class sub(Node):
	in0: Any = None
	in1: Any = None

	output = Outlet(DataType.SIGNAL)


@node
class mul(Node):
	in0: Any = None
	in1: Any = None

	output = Outlet(DataType.SIGNAL)


@node
class div(Node):
	in0: Any = None
	in1: Any = None

	output = Outlet(DataType.SIGNAL)


@node
class gt(Node):
	in0: Any = None
	in1: Any = None

	output = Outlet(DataType.SIGNAL)


@node
class lt(Node):
	in0: Any = None
	in1: Any = None

	output = Outlet(DataType.SIGNAL)


@node
class mix(Node):
	in0: Any = None
	in1: Any = None
	fac: Any = 0.5

	output = Outlet(DataType.SIGNAL)


@node
class downmix(Node):
	input: Any = None
	
	output = Outlet(DataType.SIGNAL)

@node
class sine(Node):
	frequency: Any = 440
	phase: Any = 0

	output = Outlet(DataType.SIGNAL)


@node
class square(Node):
	frequency: Any = 440
	duty: Any = 0.5
	phase: Any = 0

	output = Outlet(DataType.SIGNAL)


@node
class noise(Node):
	# Input is only used for voicing. No data read.
	voices: Any = None

	output = Outlet(DataType.SIGNAL)


@node
class random(Node):
	# Input is only used for voicing. No data read.
	input: Any = None
	
	output = Outlet(DataType.SIGNAL)


@node
class saw(Node):
	frequency: Any = 440
	phase: Any = 0

	output = Outlet(DataType.SIGNAL)


@node
class slewrate(Node):
	"""
	Increase rise and fall time
	"""
	ringing: Any = 0
	mass: Any = 0

	output = Outlet(DataType.SIGNAL)


@node
class trigger(Node):
	# The value that is to be triggered on
	value: Any = 0.0

	# Equal or more than this value outputs 1.0 on the output
	on: Any = 0.5

	# Less than this value output 0.0 on the output
	off: Any = 0.5

	# If connected, react on trigger from this source
	trigger: Optional[Any] = None

	output = Outlet(DataType.SIGNAL)


@node
class time(Node):
	voice_trigger: Any = 0.0

	# This can be modulated to modulate the speed of time
	speed: Any = 1.0

	output = Outlet(DataType.SIGNAL)


@node
class clip(Node):
	value: Any = 0.0

	minimum: Any = -1.0
	maximum: Any = 1.0

	output = Outlet(DataType.SIGNAL)


@node
class frequency(Node):
	input: Any = 0.0

	output = Outlet(DataType.SIGNAL)


@node
class one(Node):
	input: Any = None

	output = Outlet(DataType.SIGNAL)


@node
class dB(Node):
	"""Convert a dB number to a linear float.

	Reduce amplitude on audio by 9dB:
		_audio * dB(-9)
		or
		_audio * -dB(9)
	"""
	decibel: Any = 0.0

	output = Outlet(DataType.SIGNAL)


@node
class score(Node):
	"""
	Midi score written as a textfile
	"""
	path: str | None = None

	output = Outlet(DataType.MIDI)


@node
class polyphonic(Node):
	"""
	Splits up a MIDI signal into multiple voices by key
	"""
	input: Any = None
	max_voices: int = 32

	midi = Outlet(DataType.MIDI)


@node
class unison(Node):
	"""Create multiple voices

	By varying the "voices" input, more voices can be created.

	Note that voices are only deallocated if the whole following chain marks the current voice as
	removable."""
	input: Any = 0
	voices: int = 1  # For each key pressed
	max_voices: int = 32  # Destroy old voices when this limit is hit

	output = Outlet(DataType.NONE)  # The format changes dynamically


@node
class spawn(Node):
	"""
	Spawns new voices when input levels goes above 0

	The voice is held until input is 0 or lower.

	Only support voice 0.
	"""
	input: Any = None

	output = Outlet(DataType.SIGNAL)  # The format changes dynamically


@node
class hold(Node):
	input: Any = None
	condition: Any = None

	output = Outlet(DataType.NONE)  # The format changes dynamically


@node
class delay(Node):
	"""
	Delays everything

	When things come too soon, it is nice to have a delay.
	"""
	input: Any = None
	output = Outlet(DataType.SIGNAL)  # The format changes dynamically


@node
class reverb(Node):
	input: Any
	size: Any = 0.1  # "Seconds" maybe...?

	output = Outlet(DataType.SIGNAL)


@node
class audiofile(Node):
	"""
	Tries to load audio from a file
	"""
	file: str = None

	# Playback speed. Can be negative.
	speed: Any = 1.0

	# Create multiple voices of this node, allowing multiple playheads
	voices: Any = None

	output = Outlet(DataType.SIGNAL)

	def __post_init__(self):
		super().__post_init__()

		self.channel_paths = {}

		# TODO merayen retrieve sample_rate from aim project config or something?
		for path in importing.read_audio_data(self.file, 48000):
			self.channel_paths[int(path.split("-")[1])] = path


@node
class use(Node):
	"""
	Load a module (another Python file)

	The inputs and outputs are dynamically set by the module.
	"""
	path: str = None  # Relative file path


@node
class oscilloscope(OutNode):
	value: Any = 0.0
	time_div: Any = 0.01

	trigger: float = 0.1

	# Value must be lower than this value until trigging is enabled, after a trigger
	trigger_low: float = 0.0

	# Passthrough. Allows doing "out(oscilloscope(sine()))"
	output = Outlet(DataType.SIGNAL)


@node
class out(OutNode):
	input: Any = None

	# This name is exported out of the module and is visible as a port on the outside
	name: Optional[str] = None

	def __post_init__(self):
		super().__post_init__()

		if not self.name:
			context: Context = _PARSE_CONTEXT.get()
			self.name = f"unnamed_{context.unnamed_counter}"
			context.unnamed_counter += 1


class state:
	next_id = 0


def create_variable() -> str:
	state.next_id += 1
	return f"_{state.next_id}"


def load(text: str) -> Context:
	_validate_python(text)

	module_variables = {
		x.__name__: x
		for x in Node.__subclasses__() + OutNode.__subclasses__()
	}

	token = _PARSE_CONTEXT.set(Context())

	exec(text, module_variables)

	context: Context = _PARSE_CONTEXT.get()

	_PARSE_CONTEXT.reset(token)

	return context


def _validate_python(text: str) -> None:
	"""Validate that only a subset of Python is being used

	This makes Python more declarative and hopefully helps against malicious code if this were to run
	any external code."""
	import ast

	for x in ast.walk(ast.parse(text)):
		if isinstance(x, (
			ast.Div,
			ast.Gt,
			ast.Lt,
			ast.Compare,
			ast.Module,
			ast.Expr,
			ast.Load,  # TODO merayen what is this?
			ast.BinOp,  # TODO merayen what is this?
			ast.Store,  # TODO merayen what is this?
			ast.Constant,
			ast.Add,
			ast.Sub,
			ast.UnaryOp,
			ast.Attribute,
			ast.Mult,
			ast.USub,
			ast.keyword,
		)):
			pass
		elif isinstance(x, ast.Name):
			if x.id.startswith("__"):
				raise RestrictedPythonError("Can not use '__' variables")

			if x.id.startswith("_"):
				pass  # Allow "_name" variables

			elif not any(
				x.id == node.__name__
				for node in Node.__subclasses__() + OutNode.__subclasses__()
			):
				# If not "_name" symbol, require it to be a node
				raise RestrictedPythonError(f"Node not found: {x.id!r}")

		elif isinstance(x, ast.Assign):
			if [type(y) for y in x.targets] != [ast.Name]:
				raise RestrictedPythonError("Can only do simple assignments like '_a = _b' etc")

			if not x.targets[0].id.startswith("_"):
				raise RestrictedPythonError("Can only assign to variables starting with '_'")

		elif isinstance(x, (ast.Call, ast.Pass)):
			pass

		else:
			raise RestrictedPythonError(f"Element {type(x).__name__!r} can not be used")


def build_node_graph(context: Context) -> tuple[dict[int, set[int]], dict[int, Node]]:
	graph: dict[int, set[int]] = {}

	# Node <-> id() registry. It is our way of setting identifiers on the nodes
	node_ids: dict[int, Node] = {id(node): node for node in context.out_nodes}

	remaining: set[int] = set(node_ids)

	while remaining:
		node_id: int = remaining.pop()

		graph[node_id] = set()

		for node_inlet, input_value in node_ids[node_id].__dict__.items():
			if isinstance(input_value, Outlet):
				input_value = input_value.node
				if input_value is node_ids[node_id]:
					# This Outlet belongs to us, skip it
					continue

				assert input_value

			if isinstance(input_value, Node):
				assert id(input_value) != node_id, "Node depends on itself"

				# Node as input that we need to walk into
				graph[node_id].add(id(input_value))

				if id(input_value) not in node_ids:
					remaining.add(id(input_value))

				node_ids[id(input_value)] = input_value

	return graph, node_ids


def execution_order(graph: dict[int, set[int]]) -> list[int]:
	"""
	Sort the nodes in the order they need to be executed
	"""
	result = list(graph)

	i = 0
	while i < len(result):
		for dependency in graph[result[i]]:
			dependency_index = result.index(dependency)
			if dependency_index > i:  # A depedency is executed after current node
				# Move that dependency before us
				result.insert(i, result.pop(dependency_index))
				break  # Rerun at the same position, re-evaluating after our reordering
		else:  # All dependencies are executed before node
			i += 1  # We can move one position ahead

	return result


def test_a() -> None:
	@node
	class A(Node):
		in1: Any
		in0: Any
		in3: Any = object()
		in2: Any = object()

		out1 = Outlet(DataType.SIGNAL)
		out0 = Outlet(DataType.SIGNAL)
		out3 = Outlet(DataType.SIGNAL)
		out2 = Outlet(DataType.SIGNAL)

	@node
	class B(Node):
		in2: Any

		out1337 = Outlet(DataType.SIGNAL)

	for i in range(10):
		b = B(in2=42)
		a = A(1337, b)
		assert list(a._inlets.keys()) == ["in1", "in0", "in3", "in2"]
		assert list(a._inlets.values()) == [1337, b.out1337, a.in3, a.in2]
		assert list(a._outlets.keys()) == ["out1", "out0", "out3", "out2"]
		assert list(a._outlets.values()) == [a.out1, a.out0, a.out3, a.out2]


def test_execution_order() -> None:
	context = load("""
_a = midi("default")
_b = sine()
_c = add(_b, _a)
_d = add(_a, _c)
out(_d)
	""")

	graph, nodes = build_node_graph(context)
	order = execution_order(graph)
	out_node = context.out_nodes[0]
	assert isinstance(out_node, out)
	assert isinstance(out_node.input, Outlet)
	_d = out_node.input.node
	assert type(_d) == add
	_a = _d.in0.node
	assert type(_a) == midi
	_c = _d.in1.node
	assert type(_c) == add
	_b = _c.in0.node
	assert type(_b) == sine

	# Nodes with no inputs must always be first
	assert {order[0], order[1]} == {id(_a), id(_b)}

	# Check the middle ones
	assert order[2] == id(_c)

	# This always comes after _c because it depends on it
	assert order[3] == id(_d)

	# The last node is always the nodes no one depends on
	assert order[-1] == id(out_node)


def test_operator_execution_order():
	r = "out(%s)"
	for i in range(10):
		r %= f"add({i}, %s)"

	r %= "0"

	context = load(r)
	graph, node_ids = build_node_graph(context)
	order = execution_order(graph)

	assert [node_ids[x].in0 for x in order if isinstance(node_ids[x], add)] == list(range(9,-1,-1))

	# Out-node should be placed at the end
	assert isinstance(node_ids[order[-1]], out)


def test_forbidden_python() -> None:
	for x in (
		"import os",
		"from os import path",
		"__a = 0",
		"a,__b = 0",
		"for x in (1,2): pass",
		"if False: pass",
		"class A: pass",
		"__builtins__",
		"exec",
		"eval",
		"_a = exec",
		"_a = eval",
		"_a = None, exec",
		"out = 123",
		"with open('file') as f: pass",
	):
		try:
			load(x)
		except RestrictedPythonError:
			pass
		else:
			raise Exception(f"Should have restricted {x!r}")

	# Make sure these are allowed
	for x in (
		"pass",  # Used in above test, testing here to ensure not creating false negatives above
		"out()",
		"1+1",
		"1-1",
		"1*1",
		"1/1",
		"-1",
		"_a = out(sine()); _a",
	):
		try:
			load(x)
		except RestrictedPythonError:
			raise Exception(f"Should have not restricted {x!r}")


def test_gt():
	assert isinstance(sine(), Node)
	assert isinstance(sine() > sine(), gt)
	assert isinstance(sine() < sine(), lt)


if __name__ == '__main__':
	for x in dir():
		if x.startswith("test_"):
			exec(f"{x}()")
