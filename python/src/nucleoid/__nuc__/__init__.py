"""
__nuc__ module - Core statement types for Nucleoid language processing.
"""

from .__NUC__ import __NUC__
from .__ALIAS__ import __ALIAS__, build as __alias__
from .__ASSIGNMENT__ import __ASSIGNMENT__, build as __assignment__
from .__BLOCK__ import __BLOCK__, build as __block__
from .__CALL__ import __CALL__, build as __call__
from .__CLASS__ import __CLASS__, build as __class__
from .__DELETE__ import __DELETE__, build as __delete__
from .__EXPRESSION__ import __EXPRESSION__, build as __expression__
from .__FOR__ import __FOR__, build as __for__
from .__FUNCTION__ import __FUNCTION__, build as __function__
from .__IF__ import __IF__, build as __if__
from .__INSTANCE__ import __INSTANCE__, build as __instance__
from .__LET__ import __LET__, build as __let__
from .__PROPERTY__ import __PROPERTY__, build as __property__
from .__RETURN__ import __RETURN__, build as __return__
from .__THROW__ import __THROW__, build as __throw__
from .__VARIABLE__ import __VARIABLE__, build as __variable__
from .revive import revive

__all__ = [
    "__NUC__",
    "__ALIAS__",
    "__ASSIGNMENT__",
    "__BLOCK__",
    "__CALL__",
    "__CLASS__",
    "__DELETE__",
    "__EXPRESSION__",
    "__FOR__",
    "__FUNCTION__",
    "__IF__",
    "__INSTANCE__",
    "__LET__",
    "__PROPERTY__",
    "__RETURN__",
    "__THROW__",
    "__VARIABLE__",
    "__alias__",
    "__assignment__",
    "__block__",
    "__call__",
    "__class__",
    "__delete__",
    "__expression__",
    "__for__",
    "__function__",
    "__if__",
    "__instance__",
    "__let__",
    "__property__",
    "__return__",
    "__throw__",
    "__variable__",
    "revive",
]
