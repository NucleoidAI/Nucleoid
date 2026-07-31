import subprocess
import os
import uuid

def compile(python_code):
    unique_filename = f"{uuid.uuid4().hex}.py"

    try:
        with open(unique_filename, 'w') as file:
            file.write(python_code)

        subprocess.run(['transcrypt', '-n', unique_filename], check=True)

        js_filename = os.path.splitext(unique_filename)[0] + '.js'
        output_path = os.path.join('__target__', js_filename)

        with open(output_path, 'r') as file:
            js_code = file.read()

        return js_code
    finally:
        os.remove(unique_filename)
        if os.path.exists(output_path):
            os.remove(output_path)
        if os.path.exists('__target__'):
            for file in os.listdir('__target__'):
                os.remove(os.path.join('__target__', file))
            os.rmdir('__target__')
