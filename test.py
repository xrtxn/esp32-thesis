import sys

with open('src/display.rs', 'r') as f:
    lines = f.readlines()
    for i in range(250, min(300, len(lines))):
        print(lines[i], end='')
