"""
Handlers that listens for the output data for each node

E.g, an oscilloscope node that creates UI and reads plot data from the oscilloscope node.

Hardcoded to kyvi for now. Could also send outgoing node data on UDP and such, but we don't plan for
that for now.
"""
from typing import Optional
import pylab as pl
from aim.nodes import Node


class Listener:
	def __init__(self, node: Node):
		self.node = node
		self.setup()

	def setup(self):
		pass

	def receive(self, **kwargs):
		raise NotImplementedError


line_count = 10

class oscilloscope_listener(Listener):
	def setup(self):
		self.fig, self.axs = pl.subplots(1, facecolor=(0.05, 0.05, 0.1))

		self.axs.set_ylim(-1, 1)

		self.lines = {}

		self.axs.get_xaxis().set_ticks([])
		self.axs.get_yaxis().set_ticks([])
		self.axs.axis("off")

		self.fig.tight_layout()

		pl.show(block=False)

	def receive(self, voice_id: Optional[int], samples: Optional[list[float]]):
		if not pl.fignum_exists(self.fig.number):
			self.lines = None
			return  # User has closed the window

		if samples is not None:
			assert voice_id is not None

			if voice_id in self.lines and len(samples) == 0:
				for line in self.axs.lines:
					if line is self.lines[voice_id]:
						line.remove()
				self.lines.pop(voice_id)
			elif voice_id not in self.lines:
				self.lines[voice_id] = self.axs.plot(samples, antialiased=False)[0]
			else:
				self.lines[voice_id].set_ydata(samples)

			self.fig.canvas.draw()
			self.fig.canvas.flush_events()
