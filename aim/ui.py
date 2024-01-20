"""
Simple UI for aim

Meant to show things like oscilloscope and such in a grid layout window.
"""

# Remove the arguments so that kivy doesn't read them
# kivy is not a first class citizen in this project.
import sys
sys.argv = sys.argv[0:1]

from kivy.app import App
from kivy.clock import Clock
from kivy.uix.boxlayout import BoxLayout
from kivy.uix.label import Label


class AimUI(App):
	def build(self):
		layout = BoxLayout()
		layout.add_widget(Label(text="Hello, World! 0"))
		layout.add_widget(Label(text="Hello, World! 1"))

		return layout



def run_ui():
	"""
	Runs the UI

	The generator argument is code that needs to be run on each tick.
	"""
	AimUI().run()
