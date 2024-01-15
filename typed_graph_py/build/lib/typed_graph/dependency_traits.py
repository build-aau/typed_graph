from enum import Enum, EnumMeta
from pydantic import BaseModel, RootModel, model_serializer, model_validator
from typing import Any, Callable

class Enum_M(EnumMeta):
    def __new__(metacls, name: str, bases, classdict, **kwds):
        enum_class = EnumMeta.__new__(EnumMeta, name, bases, classdict, **kwds)

        # uses the values hash function
        def __hash__(self):
            return self.value.__hash__()
        setattr(enum_class, '__hash__', __hash__)

        # Compare the value of the two varients
        def __eq__(self, other):
            return self.value.__eq__(other)
        setattr(enum_class, '__eq__', __eq__)

        return enum_class

class StrEnum(str, Enum, metaclass=Enum_M):
    """
    An enum that uses str for each of its varients
    
    This allows for the specific type to be used interchangeably with a str
    """
    pass

class IntEnum(int, Enum, metaclass=Enum_M):
    """
    An enum that uses int for each of its varients
    
    This allows for the specific type to be used interchangeably with a int
    """
    pass


def make_model(base):
    """
    Create a new model type using different bases
    """

    class ModelInstance(base):
        """
        class vars:
        - tagging: Is external tagging used (default: True)
        """

        @model_serializer(mode = 'wrap')
        def _serialize(
            self, 
            default: Callable[['RustModel'], dict[str, Any]]
        ) -> dict[str, Any] | Any:
            """
            Serialize the model to a dict.

            This append an external tag to the created dict with the name of the type
            """

            # Check if tagging is disables
            if 'tagging' in self.__class_vars__ and not self.__class__.tagging:
                return default(self)

            name = self.__class__.__name__
            return {
                name: default(self)
            }
        
        @model_validator(mode = 'wrap')
        def _deserialize(
            cls, 
            d: dict[str, Any] | Any, 
            default: Callable[[dict[str, Any]], 'RustModel']
        ) -> 'RustModel':
            """
            Deserialize the model from a value

            If the value is a dict with one entry that correspond to any subclass, 
            then the subclass is deserialized instead.
            """
            if 'tagging' in cls.__class_vars__ and not cls.tagging:
                return default(d)
            
            if not isinstance(d, dict):
                return default(d)
            
            if len(d) != 1:
                return default(d)

            # Recursivly traverse sub classes to check if any of them match
            subclases = []
            subclases.extend(cls.__subclasses__())
            while subclases:
                subcls = subclases.pop()
                subclases.extend(subcls.__subclasses__())

                # Instantiate subclass
                if hasattr(subcls, 'model_validate') and subcls.__name__ in d:
                    c = subcls.model_validate(d[subcls.__name__])
                    return subcls.model_validate(d[subcls.__name__])

            if cls.__name__ in d:
                return default(d[cls.__name__])
            
            return default(d)
        
    return ModelInstance

RustModel = make_model(BaseModel)
RustRootModel = make_model(RootModel)