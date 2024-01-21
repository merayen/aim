"""
Handlers that listens for the output data for each node

E.g, an oscilloscope node that creates UI and reads plot data from the oscilloscope node.

Hardcoded to kyvi for now. Could also send outgoing node data on UDP and such, but we don't plan for
that for now.
"""
import pylab as pl
from typing import Optional

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
		import pylab as pl
		self.fig, self.axs = pl.subplots(1, facecolor=(0.05, 0.05, 0.1))

		self.axs.set_ylim(-1, 1)

		self.lines = {}

		self.axs.get_xaxis().set_ticks([])
		self.axs.get_yaxis().set_ticks([])
		self.axs.axis("off")

		self.fig.tight_layout()

		pl.show(block=False)

	def receive(self, plot_data: Optional[list[int]]):
		if not pl.fignum_exists(self.fig.number):
			return  # User has closed the window

		if plot_data is not None:
			if len(plot_data) != self.lines:
				self.axs.clear()
				self.lines = {}

			for voice_id, voice in plot_data.items():
				if voice_id not in self.lines:
					self.lines[voice_id] = self.axs.plot(voice, antialiased=False)
				else:
					self.lines[voice_id].set_ydata(voice)

			# Remove non-existent voices
			for voice_id in (x for x in self.lines if x not in self.lines):
				self.axs.lines.pop(self.axs.lines.index(self.lines[voice_id]))
				self.lines.pop(voice_id)
				# TODO merayen remove plot too?

			self.fig.canvas.draw()
			self.fig.canvas.flush_events()
