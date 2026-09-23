"""Build and test the installed distribution, isolated from source imports."""

from pathlib import Path
import subprocess
import sys
import tempfile
import tomllib
import venv
import zipfile


def main() -> None:
    project = Path(__file__).resolve().parent
    standard = project.parent
    version = tomllib.loads((project / "pyproject.toml").read_text())["project"][
        "version"
    ]
    for path, section in (
        (standard / "Cargo.toml", "package"),
        (standard / "standard.toml", "standard"),
    ):
        if tomllib.loads(path.read_text())[section]["version"] != version:
            raise ValueError(f"Python contract version differs from {path.name}")

    with tempfile.TemporaryDirectory(prefix="apss-python-contract-") as directory:
        root = Path(directory)
        environment = root / "venv"
        venv.create(environment, with_pip=True)
        python = environment / (
            "Scripts/python.exe" if sys.platform == "win32" else "bin/python"
        )

        def run(*arguments: str) -> None:
            subprocess.run([str(python), "-I", *arguments], cwd=root, check=True)

        run("-m", "pip", "install", "build")
        # Default build makes an sdist, then builds its wheel from that sdist.
        run("-m", "build", str(project), "--outdir", str(root / "dist"))
        (wheel,) = (root / "dist").glob("*.whl")
        with zipfile.ZipFile(wheel) as archive:
            if "apss_session_capture/py.typed" not in archive.namelist():
                raise ValueError("wheel is missing its typing marker")
        run("-m", "pip", "install", str(wheel))
        run("-m", "pip", "check")
        run("-m", "unittest", "discover", "-s", str(project / "tests"), "-v")


if __name__ == "__main__":
    main()
