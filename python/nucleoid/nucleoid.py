from .parse import parse
from .process import process

class   Nucleoid:
    def __init__(self):
        print("Nucleoid object created")

        #clear all the graph and state etc(for later)


    def run(self, statement):
        if isinstance(statement, str):
            print("Running statement: ", statement)
            parsed_tree = parse(statement)
            out = process(parsed_tree)
            return out
