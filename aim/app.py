import os
import random
import sys
import shutil

blocks = [[]]

class Node:
    """Represents a node, which is the units the synth is made of"""

    node_registry = {}

    def __init_subclass__(cls):
        assert getattr(cls, "name", None)
        Node.node_registry[cls.name] = cls

    def __init__(self, nick=None):
        self.nick = nick
        self.properties = {}

    def __repr__(self):
        return f"Node(name={self.name},	nick={self.nick}, properties={self.properties})"

    def __str__(self):
        assert not any(" " in x for x in self.properties), "Space found in node property"
        return f"# {self.name} {self.nick}\n" + "\n".join(f"{k} {v}\n" for k, v in self.properties.items())


class Nodes:
    def __init__(self):
        super().__init__()
        self.ids = set()
        self.nodes = {}

    def add(self, name, nick=None):
        nick = nick or "_" + "".join(random.choice("abcdefghjklmnpqrstvwxyz") for _ in range(10))
        assert nick not in self.nodes, f"Node {nick} already added to list"
        node = Node.node_registry[name](nick)
        self.nodes[nick] = node
        return node

    def __len__(self):
        return len(self.nodes)

    def __repr__(self):
        return repr(self.nodes)

    def __str__(self):
        return "\n".join(str(node) for node in self.nodes.values())

    def __getitem__(self, *a, **b):
        return self.nodes.__getitem__(*a, **b)


class Score(Node):
    name = "score"


class Sine(Node):
    name = "sin"


class Out(Node):
    name = "out"


def parse(path) -> Nodes:
    def fail(text=None):
        print(f"Line {i+1}")
        exit(1)

    def parse_command(line):
        pass

    nodes = Nodes()

    with open(path) as f:
        last_node = None
        for i, line in enumerate(f.readlines()):
            line = line.strip("\n")
            if line.startswith("# "):
                header = line.strip().split(" ", 1)[1]
                name = header.split(" ", 1)[0]
                nick = (header.split(" ", 1)[1:] or [None])[0]
                last_node = nodes.add(name, nick)
            elif line.strip():  # Everything is a property
                key, value = line.split(" ", 1)
                last_node.properties[key] = value

    return nodes


def write(path, nodes: Nodes):
    path = os.path.abspath(path)
    with open(path, "w") as f:
        f.write(str(nodes))
