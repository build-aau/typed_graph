from enum import Enum, EnumMeta
from pydantic import BaseModel, RootModel, model_serializer, model_validator
from typing import Any, Callable
from pydantic._internal._model_construction import ModelMetaclass
import inspect

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
            default: Callable   [['RustModel'], dict[str, Any]]
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

class NestedEnumMeta(ModelMetaclass):
    def __new__(metacls, name, bases, class_dct, *args, **kwargs):
        """
        Create a new enum class with a number og varients as attributes
        Each varient has their own class 
        that inherits all the base classes of the enum except for its pydantic model
        """

        # Retrieve varient annotations
        annotations = None
        for k, v in class_dct.items():
            if k == '__annotations__':
                annotations = v
        # Stop the varients from being made as fields in the enum base model
        if '__annotations__' in class_dct:
            del class_dct['__annotations__']
        
        enum_class = super().__new__(metacls, name, bases, class_dct, *args, **kwargs)

        # Create a constructor on the enum that prevents it from being initialized
        def __new__(self, *args, **kwarg):
            raise Exception(f'Can\'t initialize enum type {name}')
        setattr(enum_class, '__new__', __new__)

        # Find all bases clases that the varients should also inherit
        varient_bases = []
        for enum_base in bases:
            if enum_base.__name__ != 'NestedEnum' and not issubclass(enum_base, BaseModel):
                varient_bases.append(enum_base)

        enum_varients = {}

        # Create varients if any are provided
        if annotations:
            for varient_name, varient_type in annotations.items():
                varient_class = NestedEnumMeta.create_varient(varient_name, varient_type, varient_bases, class_dct)

                setattr(enum_class, varient_name, varient_class)
                enum_varients[varient_name] = varient_class

        setattr(enum_class, '_members', enum_varients)
        return enum_class
    
    @staticmethod
    def create_varient(varient_name, varient_type, varient_bases, class_dct, ):
        varient_type_name = f"{class_dct['__qualname__']}.{varient_name}"
                
        if varient_type == str:
            # Handle unit varients
            class_bases = [RootModel, *varient_bases]
            variation_class = ModelMetaclass.__new__(
                ModelMetaclass, 
                varient_type_name, 
                (RootModel, ), 
                {
                    '__module__': class_dct['__module__'], 
                    '__qualname__': varient_type_name,
                    '__annotations__': { 
                        'root': str,
                    },
                }
            )

            return variation_class(varient_name)

        elif isinstance(varient_type, dict):
            # Handle struct varients
            class_bases = [RustModel, *varient_bases]

            varient_dict = {
                '__module__': class_dct['__module__'], 
                '__qualname__': varient_type_name,
                '__annotations__': varient_type
            }

            # pass information about generic along
            if '__orig_bases__' in class_dct:
                varient_dict['__orig_bases__'] = class_dct['__orig_bases__']

            variation_class = ModelMetaclass.__new__(
                ModelMetaclass, 
                varient_type_name, 
                tuple(class_bases), 
                varient_dict
            )

            return variation_class
        else:
            raise Exception(f"Unsupported varient type {varient_type} expected {str(str)} or {str(dict)}")
    
class NestedEnum(BaseModel, metaclass=NestedEnumMeta):
        
    @model_validator(mode = 'wrap')
    def _deserialize(
        cls, 
        d: dict[str, Any] | Any, 
        default: Callable[[dict[str, Any]], 'RustModel']
    ) -> 'RustModel':
        # Handle unit varients
        if isinstance(d, str) and  d in cls._members:
            varient = cls._members[d]
            if not inspect.isclass(varient):
                return varient.model_validate(d)

        # If it is neither, then it must just be the enum
        if not isinstance(d, dict):
            return default(d)

        if len(d) != 1:
            return default(d)

        # Handle dict varient            
        varient_name = next(iter(d.keys()))
        if varient_name in cls._members:
            varient = cls._members[varient_name]
            if inspect.isclass(varient):
                return varient.model_validate(d[varient_name])

        return default(d)

    @classmethod
    def __class_getitem__(cls, ty):
        instance = super().__class_getitem__(ty)

        # We only populate _members if it is empty
        # This is because Generic reuses the same class
        if not hasattr(instance, '_members') or not instance._members :
            instance._members = {}

            # We now need to propergate the generics from the enum to its varients
            for name, _ in cls._members.items():
                
                varient_instance = getattr(instance, name)

                # Do not propergate generics to unit enums
                if inspect.isclass(varient_instance):
                    varient_instance = varient_instance[ty]
                
                # Update varients on instance class
                setattr(instance, name, varient_instance)
                instance._members[name] = varient_instance

        return instance