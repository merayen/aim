"""
Handlers that listens for the output data for each node

E.g, an oscilloscope node that creates UI and reads plot data from the oscilloscope node.
"""
from typing import Optional

from aim.nodes import Node


class Listener:
	def __init__(self, node: Node):
		self.node = node

	def receive(self, **kwargs):
		raise NotImplementedError


class oscilloscope_listener(Listener):
	def receive(self, plot_data: Optional[list[int]]):
		pass
