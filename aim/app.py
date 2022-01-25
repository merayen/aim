import shutil
import os
import random
import sys
import io
import shutil

blocks = [[]]

ids = set()


class Node:
    """Represents a node, which is the units the synth is made of"""

    node_registry = {}

    def __init_subclass__(cls):
        assert getattr(cls, "name", None)
        Node.node_registry[cls.name] = cls.__class__

    def __init__(self, nick=None):
        self.nick = nick
        self.properties = {}

    def __repr__(self):
        return f"Node(name={self.name},	nick={self.nick}, properties={self.properties})"

    def __str__(self):
        result = io.StringIO()
        result.write(f"# {self.name} {self.nick}")
        return result.getvalue()


class Nodes:
    def __init__(self):
        super().__init__()
        self.ids = set()
        self.nodes = []

    def add(self, name, nick=None):
        nick = nick or "_" + "".join(random.choice("abcdefghjklmnpqrstvwxyz") for _ in range(10))
        assert nick not in (x.nick for x in self.nodes), f"Node {nick} already added to list"
        self.nodes.append(Node.node_registry[name](nick))

    def __len__(self):
        return len(self.nodes)

    def __repr__(self):
        return repr(self.nodes)

    def __str__(self):
        result = io.StringIO()
        for node in self.nodes:
            result.write(str(node))
        return result


class Score(Node):
    name = "score"


class Sine(Node):
    name = "sin"


class Out(Node):
    name = "out"


def parse(path) -> Nodes:
    def fail(text=None):
        print(f"Line {i+1}")

    nodes = Nodes()

    with open(path) as f:
        for i, line in enumerate(f.readlines()):
            line = line.strip("\n")
            if line.startswith("# "):
                header = line.split(" ", 1)[1]
                name = header.split(" ", 1)[0]
                nick = (header.split(" ", 1)[1:] or [None])[0]
                print(name, nick)
                nodes.add(name, nick)

    return nodes


def write(path, nodes: Nodes):
    path = os.path.abspath(path)
    with open(path) as f:
        f.write(str(nodes))
