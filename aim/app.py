import os
import random
import shutil
import subprocess
import sys

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

    def on_parse(self):
        pass


class Nodes:
    def __init__(self):
        super().__init__()
        self.ids = set()
        self.nodes = {}

    def add(self, name, nick=None):
        nick = nick or "ID" + "".join(random.choice("abcdefghjklmnpqrstvwxyz") for _ in range(10))
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

    def __iter__(self):
        return self.nodes.values().__iter__()


class Score(Node):
    name = "score"

    def on_parse(self):
        print("Score got properties", self.properties)
        if "file" in self.properties:
            with open(self.properties["file"]) as f:
                data = f.read().strip()

            if not data or 1:
                # Score is empty, make a blank one
                data = " ".join("ABCDEFG"[i % 7] for i in range(7*6)) + "\n"
                data += " ".join(str(i // 7) for i in range(7*6)) + "\n"
                data += "-" * (7*6*2) + "\n"
                for i in range(100):
                    data += " |" * (7*6) + "\n"

            # Parse the file

            # Look for commands

            with open(self.properties["file"], "w") as f:
                f.write(data)

            import pathlib
            pathlib.Path(self.properties["file"]).touch()


class Sine(Node):
    name = "sin"


class Out(Node):
    name = "out"


def parse_folder(path):  # TODO merayen Support parsing whole folders
    # Probably start with main.txt, or whatever that is "main" or startswith "main.", and require only 1
    pass


def parse(path) -> Nodes:
    def fail(text=None):
        print(f"{path}:{i+1}" + (f" {text}" if text else ""))
        exit(1)

    def parse_command(line):
        pass

    nodes = Nodes()

    with open(path) as f:
        last_node = None
        for i, line in enumerate(f.readlines()):
            line = line.strip()
            if not line:
                pass
            elif line.startswith("# "):
                if last_node:
                    last_node.on_parse()

                header = line.split(" ", 1)[1]
                name = header.split(" ", 1)[0]
                nick = (header.split(" ", 1)[1:] or [None])[0]
                last_node = nodes.add(name, nick)
            elif len(line.split(" ", 1)) == 2:
                if not last_node:
                    fail(f"Expected node declaration, but got: {line}")
                key, value = line.split(" ", 1)
                last_node.properties[key] = value
            else:
                fail(f"Can't parse: {line}")

    return nodes


def write(path, nodes: Nodes):
    path = os.path.abspath(path)
    with open(path, "w") as f:
        f.write(str(nodes))

def transpile(output_path: str, nodes: Nodes) -> str:
    """Transpile to C"""
    from .transpiler import Transpiler
    with open(output_path, "w") as f:
        f.write(str(Transpiler(nodes)))

def llvm_compile(c_path: str):
    filename = "output"
    subprocess.run(["clang-13", "-o", filename, c_path.encode("utf-8")], check=True)
    return os.path.split(c_path)[0] + os.path.sep + filename

def run(bin_path):
    subprocess.run([bin_path], check=True)
