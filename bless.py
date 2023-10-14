import os
import re
from pathlib import Path
import shutil
import argparse

root = Path(__file__).parent
resources = root / 'resources'

parse = argparse.ArgumentParser()
parse.add_argument('mod_glob', type=str)
parse.add_argument('tag_glob', type=str)

args = parse.parse_args()

actual_postfix = "-actual.json.xz"
expect_postfix = "-expect.json.xz"
mod_glob = args.mod_glob
tag_glob = args.tag_glob + actual_postfix

print(f'Mod Glob: "{mod_glob}", Tag Glob: "{tag_glob}"')

to_bless = []
for mod in resources.glob(mod_glob):
    if not mod.is_dir:
        continue
    print(mod)
    for tag in mod.glob(tag_glob):
        to_bless.append(tag)

print('Bless?')
print('\n'.join(str(path.relative_to(root)) for path in to_bless))

ans = input('[y/N]')
if ans in ('y', 'Y'):
    for path in to_bless:
        move_to = path.parent / path.name.replace(actual_postfix, expect_postfix)
        shutil.copy(path, move_to)
else:
    print("Aborted")


