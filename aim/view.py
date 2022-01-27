"""Draw stuff on screen, or do something else."""


class View:
    def __init__(self, text=""):
        self.width = 0
        self.height = 0
        self.attributes: list[str]
        self.commands: list[str]
        self.__read(text)

    def __read(self, text: str):
        """Read file, attributes and commands"""
        commands = []
        attributes = []
        width = 0
        height = 0
        for line in text.splitlines():

            # Measure the widthest line
            width = max(len(line), width)

            if line.strip().startswith(":"):  # Special line that contains command
                commands.append(line.split(":", 1)[1].strip())
                continue

            for i, a in enumerate(line.split("[")):
                if i:
                    attributes.append(a.split("]", 1)[0])

            height += 1

        # All good in hood
        self.attributes = attributes
        self.commands = commands
        self.width = width
        self.height = height

    def resize(self, width, height):
        self.width = width
        self.height = height
        raise NotImplementedError("rebuild the buffer")  # TODO merayen

    def draw(self, buffer: "View", x: int, y: int):
        assert isinstance(buffer, View)

        required_width = buffer.width + x
        required_height = buffer.height + y

        raise NotImplementedError  # TODO merayen

        for i, x in enumerate(buffer):
            pass
