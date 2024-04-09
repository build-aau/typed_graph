from setuptools import setup
import toml

version = ''
with open('../Cargo.toml', 'r') as f:
    cargo = toml.load(f)
    version = cargo['package']['version']
    
setup(
    version=version
)