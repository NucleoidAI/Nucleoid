import ast

def parse(statement):
    print("Parsing statement: ", statement)
    tree = ast.parse(statement)
    return tree