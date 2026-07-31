
project = 'IronCalc'
author = 'Nicolás Hatcher'

release = '0.8.3'
version = '0.8'

extensions = ['sphinx.ext.autodoc', 'sphinx.ext.napoleon']

templates_path = ['_templates']

exclude_patterns = []

html_theme = 'furo'
html_title = 'IronCalc Python'
html_static_path = ['_static']
html_css_files = ['custom.css']

# the logo files are copied into _static by the build scripts (see
# docs/ironcalc.dev/build-docs.sh) from assets/logo/svg
html_theme_options = {
    'light_logo': 'logo-light.svg',
    'dark_logo': 'logo-dark.svg',
    'sidebar_hide_name': True,
    'light_css_variables': {
        'color-brand-primary': '#d68742',
        'color-brand-content': '#d68742',
        'color-brand-visited': '#d68742',
        'color-background-primary': '#fcfcfc',
    },
    'dark_css_variables': {
        'color-brand-primary': '#f2994a',
        'color-brand-content': '#f2994a',
        'color-brand-visited': '#f2994a',
    },
}
