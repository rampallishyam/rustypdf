from setuptools import setup

setup(
    name="rustypdf",
    version="0.1.0",
    description="CLI tool to compress and merge PDFs (Rust backend via PyO3)",
    author="Shyam Sundar",
    author_email="rampallishyam2@gmail.com",
    license="MIT",
    packages=["rustypdf"],
    python_requires=">=3.8",
    install_requires=["pypdf>=4,<5"],
    entry_points={
        "console_scripts": [
            "rustypdf=rustypdf.__main__:main",
        ],
    },
    classifiers=[
        "Programming Language :: Python",
        "Programming Language :: Python :: 3",
        "Programming Language :: Rust",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
    ],
)
