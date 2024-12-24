from .logger import setup_logger


class Calculator:
    def __init__(self):
        self.result = 0
        self.logger = setup_logger(__name__)
        self.logger.info("Calculator initialized")

    def add(self, x, y):
        self.logger.debug(f"Adding numbers: {x} + {y}")
        self.result = x + y
        self.logger.info(f"Addition result: {self.result}")
        return self.result

    def subtract(self, x, y):
        self.logger.debug(f"Subtracting numbers: {x} - {y}")
        self.result = x - y
        self.logger.info(f"Subtraction result: {self.result}")
        return self.result

    def multiply(self, x, y):
        self.logger.debug(f"Multiplying numbers: {x} * {y}")
        self.result = x * y
        self.logger.info(f"Multiplication result: {self.result}")
        return self.result

    def divide(self, x, y):
        self.logger.debug(f"Dividing numbers: {x} / {y}")
        if y == 0:
            self.logger.error("Division by zero attempted")
            raise ValueError("Division by zero is not allowed")
        self.result = x / y
        self.logger.info(f"Division result: {self.result}")
        return self.result
