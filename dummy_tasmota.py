# /// script
# requires-python = "==3.13"
# dependencies = ["fastapi[standard]"]
# ///

"""
dummy tasmota server to test outgoing http requests from esp32

```bash
uv run dummy_tasmota.py
```
"""

from fastapi import FastAPI
import uvicorn

APP = FastAPI()


@APP.post("/cm")
def command(cmnd: str):
    return {"cmnd": cmnd}


if __name__ == "__main__":
    uvicorn.run("dummy_tasmota:APP", port=80, host="0.0.0.0")
