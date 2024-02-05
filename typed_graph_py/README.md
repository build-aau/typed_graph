# TypedGraph
Is a staticly typed graph library.
This is a cross compatible port of [TypedGraph][typed_graph-crates-io].

[![pypi](https://img.shields.io/pypi/v/typed_graph.svg)](https://pypi.org/pypi/typed_graph/)

TypedGraph is build to provide an easy to use interface for manipulating an maintaining graphs with a rigid schema.

## Getting started
install using
```
pip install typed_graph
```

see the [example folder][examples-git] for information on how to create your first project using TypedGraph.

> Note that making and maintianing large static graphs can be time consuming, so using like [typed_graph_cli][typed_graph_cli-git] will make the library significantly easier to maintian 

## Development
To run a local instance of the library:
```
.../typed_graph_py> pip install -e .
```

This will import the library using a symlink, so changes in the directory will be propegated to the python installation

Now it can be used as normal
```
import type_graph
```

[typed_graph-crates-io]: https://crates.io/crates/typed_graph
[examples-git]: https://github.com/build-aau/typed_graph/tree/master/typed_graph_py/examples "example folder in git"
[typed_graph_cli-git]: https://github.com/build-aau/typed_graph_cli "typed_graph_cli in git"

# Build release
First install twine
```pip install twine```  
Then to build the release run:
```python setup.py sdist bdist_wheel```  
Then test that the build is valid:
```twine check dist/*```  
And finally upload to PyPi with:
```twine upload dist/*```  

> To avoid having to retype your initials everytime setup a file in `$HOME/.pypirc`
```
[pypi]
username = <username>
password = <password>
```

> Note: your password will be stored in plain text!

> This is also compatible with PyPi access tokens
```
[pypi]
username = __token__
password = pypi-...
```

