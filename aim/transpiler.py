"""Transpiles nodes to C"""
from .app import Nodes
from .c_nodes import CNode


class Transpiler:
    def __init__(self, nodes: Nodes):
        self.cnodes = [CNode.node_registry[node.name](node) for node in nodes]

    def __str__(self):
        headers = "\n\n".join(node.generate_header() for node in self.cnodes)
        bodies = "\n\n".join(node.generate_body() for node in self.cnodes)
        return f"""
#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#include <string.h>
#include <time.h>
#include <pthread.h>
#include <stdbool.h>
#include <unistd.h>

%(headers)s

%(bodies)s

int main() {{
    printf("Hello");
    return 0;
}}""".strip() % {"headers": headers, "bodies": bodies}
