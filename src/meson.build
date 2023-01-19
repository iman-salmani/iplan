pkgdatadir = join_paths(get_option('prefix'), get_option('datadir'), meson.project_name())
moduledir = join_paths(pkgdatadir, 'iplan')
pkgdatadir
gnome = import('gnome')

gnome.compile_resources('iplan',
  'iplan.gresource.xml',
  gresource_bundle: true,
  install: true,
  install_dir: pkgdatadir,
)

python = import('python')

conf = configuration_data()
conf.set('PYTHON', python.find_installation('python3').path())
conf.set('VERSION', meson.project_version())
conf.set('localedir', join_paths(get_option('prefix'), get_option('localedir')))
conf.set('pkgdatadir', pkgdatadir)

configure_file(
  input: 'iplan.in',
  output: 'iplan',
  configuration: conf,
  install: true,
  install_dir: get_option('bindir')
)

iplan_sources = [
  '__init__.py',
  'main.py',
]

views_sources = [
  'views/window.py',
  'views/utility_pane.py',
  'views/projects_menu.py',
  'views/primary_menu.py',
  'views/page.py',
  'views/page_header.py',
  'views/page_item.py',
]

install_data(iplan_sources, install_dir: moduledir)
install_data(views_sources, install_dir: join_paths(moduledir, 'views'))
install_data(['database/database.py'], install_dir: join_paths(moduledir, 'database'))
