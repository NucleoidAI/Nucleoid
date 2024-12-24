import logging


def setup_logger(name: str) -> logging.Logger:
    """
    Set up a logger with console handler only

    Args:
        name: The name of the logger

    Returns:
        logging.Logger: Configured logger instance
    """
    logger = logging.getLogger(name)
    logger.setLevel(logging.DEBUG)

    if logger.handlers:
        return logger

    console_formatter = logging.Formatter("%(levelname)s - %(message)s")

    console_handler = logging.StreamHandler()
    console_handler.setLevel(logging.INFO)
    console_handler.setFormatter(console_formatter)

    logger.addHandler(console_handler)

    return logger
