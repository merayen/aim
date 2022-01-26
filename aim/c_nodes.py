"""Implements nodes that generates C code"""
from .app import Node

class CNode:
    node_registry = {}

    def __init_subclass__(cls):
        assert getattr(cls, "name", None)
        CNode.node_registry[cls.name] = cls

    def __init__(self, node: Node):
        self.node = node

    def generate_header(self):
        return f"// Would generate header code for node {self.node.name}"

    def generate_body(self):
        return f"// Would generate body code for node {self.node.name}"

class CScore(CNode):
    name = "score"

class CSin(CNode):
    name = "sin"

class CScore(CNode):
    name = "out"

class CPoly(CNode):
    name = "poly"
