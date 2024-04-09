rmdir /S /Q "typed_graph_py/dist"
rmdir /S /Q "target/py/win"
mkdir "target/py/win"

cd typed_graph_py
python3 setup.py sdist bdist_wheel
cd dist
7z a "../../target/py/win/typed_graph_py.7z" typed_graph-*.whl
cd ../..

