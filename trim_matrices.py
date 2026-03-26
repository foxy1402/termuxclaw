import os
import re

def process_workflow(filepath):
    with open(filepath, 'r', encoding='utf-8') as f:
        text = f.read()

    # 1. Strip the docker job
    text = re.sub(r'\s+docker:.*?(?=\s+[a-z0-9_-]+:|\Z)', '\n', text, flags=re.DOTALL)

    # 2. Trim the matrix
    # The matrix starts at `        include:`
    # Followed by a list of OSes
    # We want to match the include list and replace it with just the android one.
    android_target = """
        include:
          - os: ubuntu-latest
            target: aarch64-linux-android
            artifact: zeroclaw
            ext: tar.gz
            ndk: true"""
            
    # Regex to find `include:` up to the next `    steps:`
    text = re.sub(r'        include:.*?    steps:', android_target + '\n    steps:', text, flags=re.DOTALL)

    # 3. There might be some lingering skip_prometheus or cross_compiler steps, but the android one doesn't use them so who cares.
    
    # Write back
    with open(filepath, 'w', encoding='utf-8') as f:
        f.write(text)

files = [
    '.github/workflows/release-beta-on-push.yml',
    '.github/workflows/release-stable-manual.yml'
]

for file in files:
    if os.path.exists(file):
        process_workflow(file)
        
print("Updated release workflows.")

# Now update checks-on-pr.yml
pr_file = '.github/workflows/checks-on-pr.yml'
with open(pr_file, 'r', encoding='utf-8') as f:
    text = f.read()

# Only keep x86_64-unknown-linux-gnu. Also remove check-32bit.
pr_matrix = """
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu"""
text = re.sub(r'        include:.*?    steps:', pr_matrix + '\n    steps:', text, flags=re.DOTALL)

# Also remove check-32bit
text = re.sub(r'\s+check-32bit:.*?(?=\s+gate:|\Z)', '\n', text, flags=re.DOTALL)

# Update gate needs
text = text.replace('needs: [lint, test, build, security, check-32bit]', 'needs: [lint, test, build, security]')

with open(pr_file, 'w', encoding='utf-8') as f:
    f.write(text)

print("Updated PR checks.")
