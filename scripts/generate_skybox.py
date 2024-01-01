import random
import subprocess
import os
import pathlib
import cairosvg

PARENT_DIR = str(pathlib.Path(__file__).parent)

NUM_STARS = 1000
INKSCAPE_EXECUTABLE = "inkscape"
OUTPUT_PATH = f"{PARENT_DIR}/../assets/skybox2.svg"
OUTPUT_WIDTH = 2000
ASPECT_RATIO = 1 / 6


output_file = open("skybox_output.svg", "w")


content = ""

for _ in range(NUM_STARS):
    content += f"""
    <circle
        cx="{random.uniform(0, OUTPUT_WIDTH)}mm"
        cy="{random.uniform(0, OUTPUT_WIDTH / ASPECT_RATIO)}mm"
        r="{random.uniform(2.0, 2.8)}mm"
        fill="#ffffff"
        fill-opacity="{random.uniform(0.7, 1.0)}"
        stroke="none"/>
    """


output_file.write(
f"""<?xml version="1.0" encoding="UTF-8" standalone="no"?>

<svg
    width="{OUTPUT_WIDTH}mm"
    height="{OUTPUT_WIDTH / ASPECT_RATIO}mm"
    version="1.1"
    id="svg1"
    sodipodi:docname="skybox.svg"
    xmlns:sodipodi="http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd"
    xmlns="http://www.w3.org/2000/svg"
    xmlns:svg="http://www.w3.org/2000/svg">

    <rect
       style="fill:#191970;fill-opacity:1;"
       width="{OUTPUT_WIDTH}mm"
       height="{OUTPUT_WIDTH / ASPECT_RATIO}mm"
       x="0"
       y="0" />

    {content}

</svg>
"""
)

output_file.close()


cairosvg.svg2png(
    url=f"{PARENT_DIR}/skybox_output.svg", 
    write_to=f"{PARENT_DIR}/../assets/skybox.png",
    output_width=OUTPUT_WIDTH
)

os.remove(f"{PARENT_DIR}/skybox_output.svg")