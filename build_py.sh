#!/bin/bash
set -e

rm -rf "typed_graph_py/dist"
rm -rf "target/py/manylinux"
mkdir -p "target/py/manylinux"

cd typed_graph_py
python3 setup.py sdist bdist_wheel
cd dist
7z a "../../target/py/manylinux/typed_graph_py.7z" typed_graph-*.whl
cd ../..

