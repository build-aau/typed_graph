from enum import Enum, EnumMeta
from pydantic import BaseModel, RootModel, model_serializer, model_validator, ConfigDict, Discriminator
from typing import Any, Callable, Dict, Type, Union, Literal
from pydantic._internal._model_construction import ModelMetaclass
import inspect

from typing import Any, Dict, List, Type, ClassVar, Annotated

from pydantic_core import core_schema
from pydantic.json_schema import JsonSchemaValue, update_json_schema, GenerateJsonSchema

from pydantic import BaseModel, GetJsonSchemaHandler, Tag

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

        def __init__(self, *args, **kwargs):
            if self.__class__.is_tagging():
                tag_name = self.__class__.get_tag_name()
                kwargs = {tag_name: kwargs}

            super().__init__(*args, **kwargs)

        # model_config = ConfigDict(json_schema_extra=_update_model_schema)

        @classmethod
        def is_tagging(cls):
            return 'tagging' not in cls.__class_vars__ or cls.tagging == True

        @classmethod
        def get_tag_name(cls):
            if 'tag_name' in cls.__class_vars__:
                return cls.tag_name
            else:
                return cls.__name__

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
            if not self.is_tagging():
                return default(self)

            name = self.__class__.get_tag_name()
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
            
            if isinstance(d, cls):
                return d
            
            if not cls.is_tagging():
                return default(d)
            if isinstance(d, cls):
                return d
            
            if not isinstance(d, dict):
                raise ValueError('value must be dict')
            
            if len(d) != 1:
                raise ValueError('empty dict cannot contain literal')
            
            cls_tag = cls.get_tag_name()
            if cls_tag in d:
                return default(d[cls_tag])
            else:
                raise ValueError(f'failed to find tag "{cls_tag}" in dict')    
        
        @classmethod
        def __get_pydantic_json_schema__(
            cls, core_schema: core_schema.CoreSchema, handler: GetJsonSchemaHandler
        ) -> JsonSchemaValue:
            json_schema = handler(core_schema)
            json_schema = handler.resolve_ref_schema(json_schema)

            # Apply the external tagging
            if cls.is_tagging():
                current_schema = {}
                current_schema.update(json_schema)
                json_schema.clear()

                tag_name = cls.get_tag_name()

                current_schema = {
                    'properties': {
                        tag_name: current_schema
                    },
                    'required': [tag_name],
                    'title': cls.__name__ + '_tag',
                    'additionalProperties': False,
                    'type': 'object'
                }

                json_schema.update(current_schema)

            return json_schema
        
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

        # Retrieve list of varients
        annotations = None
        for k, v in class_dct.items():
            if k == '__annotations__':
                annotations = v
        
        # Stop the varients from being made as fields in the enum base model
        if '__annotations__' in class_dct:
            del class_dct['__annotations__']

        # We propergate all base classes from the enum onto its varients
        # This allows for generics to be specified on the enum and then passed to the varients
        varient_bases = []
        for enum_base in bases:
            if enum_base.__name__ != 'NestedEnum' and not issubclass(enum_base, BaseModel):
                varient_bases.append(enum_base)

        enum_varients = {}

        # Create all the varients
        if annotations:
            for varient_name, varient_type in annotations.items():
                varient_class = NestedEnumMeta.create_varient(varient_name, varient_type, varient_bases, class_dct)
                enum_varients[varient_name] = varient_class

        # Even though we do not allow the enum to be initialized 
        # we still have to tell it what data we expect
        types = []
        for k, ty in enum_varients.items():
            if not callable(ty):
                types.append(Annotated[type(ty), Tag(k)])
            else:
                types.append(Annotated[ty, Tag(k)])

        if len(types) == 0:        
            class_dct['__annotations__'] = {
                'root': None
            }
        elif len(types) == 1:
            class_dct['__annotations__'] = {
                'root': types[0]
            }
        else:
            def get_tag(d):
                if isinstance(d, dict):
                    if len(d) == 1: 
                        return next(d.keys())
                    return ValueError('invalid enum varient length')
                elif isinstance(d, str):
                    return d
                else:
                    return ValueError('expected enum varient')
                
            class_dct['__annotations__'] = {
                'root': Annotated[
                    Union.__getitem__(tuple(types)),
                    Discriminator(get_tag)
                ]
            }

        enum_class = super().__new__(
            metacls, 
            name, 
            bases, 
            class_dct, 
            *args, 
            **kwargs
        )

        # We do not want people to initalize the enum by mistake as it does not contain any data
        def __new__(self, *args, **kwarg):
            raise Exception(f'Can\'t initialize enum type {name}')
        setattr(enum_class, '__new__', __new__)

        # Store all the varients
        for varient_name, varient_class in enum_varients.items():
            setattr(enum_class, varient_name, varient_class)
        setattr(enum_class, '_members', enum_varients)

        for varient in enum_varients.values():
            setattr(varient, '_parent_members', enum_varients)

        return enum_class
    
    @staticmethod
    def create_varient(varient_name, varient_type, varient_bases, class_dct, ):
        varient_type_name = f"{class_dct['__qualname__']}.{varient_name}"
                
        if varient_type == str:
            # Unit varients are just a wrapper around a Literal

            class_bases = [RootModel, *varient_bases]
            class UnitVarient(RootModel):
                root: Literal.__getitem__((varient_name, ))

            variation_class = ModelMetaclass.__new__(
                ModelMetaclass, 
                varient_type_name, 
                (UnitVarient, ), 
                {
                    '__module__': class_dct['__module__'], 
                    '__qualname__': varient_type_name,
                }
            )

            return variation_class(varient_name)

        elif isinstance(varient_type, dict):
            # For anonymous struct we create an entire class to store all the fields

            # Handle struct varients
            class_bases = [RustModel, *varient_bases]

            varient_type['tag_name'] = ClassVar[str]
            varient_dict = {
                '__module__': class_dct['__module__'], 
                '__qualname__': varient_type_name,
                '__annotations__': varient_type,
                'tag_name': varient_name
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
            # For all other types we create a light wrapper

            # The internal type is made available through inheritance
            class_bases = [varient_type, RustModel, *varient_bases]
            
            varient_dict = {
                '__module__': class_dct['__module__'], 
                '__qualname__': varient_type_name,
                '__annotations__': {
                    'tag_name': ClassVar[str],
                    'tagging': ClassVar[bool]
                },
                'tagging': True,
                'tag_name': varient_name
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
    
class NestedEnum(RootModel, metaclass=NestedEnumMeta):
    # The root is set by NestedEnumMeta

    @model_validator(mode = 'wrap')
    def _deserialize(
        cls, 
        d: dict[str, Any] | Any, 
        default: Callable[[dict[str, Any]], 'RustModel']
    ) -> 'RustModel':
        # Handle unit varients
        if isinstance(d, str):
            if d in cls._members:
                varient = cls._members[d]
                if not inspect.isclass(varient):
                    return varient.model_validate(d)
            raise ValueError(f'invalid unit varient {d}')

        # If it is neither, then it must just be the enum
        if not isinstance(d, dict):
            for varient in cls._members.values():
                if not inspect.isclass(varient) and d == varient or isinstance(d, varient):
                    return d
            raise ValueError(f'invalid varient initialization for {type(d)}')

        if len(d) != 1:
            return default(d)

        # Handle dict varient            
        varient_name = next(iter(d.keys()))
        if varient_name in cls._members:
            varient = cls._members[varient_name]
            if inspect.isclass(varient):
                return varient.model_validate(d)
            
        return default(d)

    @model_serializer(mode = 'wrap')
    def _serialize(
        self, 
        default: Callable   [['NestedEnum'], dict[str, Any]]
    ) -> dict[str, Any] | Any:
        # Figure out which varient 
        if hasattr(self, '_parent_members'):
            for varient in self._parent_members.values():
                if not inspect.isclass(varient) and self == varient or isinstance(self, varient):
                    return self.model_dump()
        raise ValueError(f'failed to match {self} to a varient')

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