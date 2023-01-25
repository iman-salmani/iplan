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

views_sidebar_sources = [
  'views/sidebar/sidebar.py',
  'views/sidebar/sidebar_projects.py',
  'views/sidebar/sidebar_project.py',
]
views_project_sources = [
  'views/project/project_header.py',
  'views/project/project_lists.py',
  'views/project/project_list.py',
  'views/project/project_list_task.py',
  'views/project/project_list_delete_dialog.py',
  'views/project/project_edit_window.py',
  'views/project/project_delete_dialog.py',
]

views_search_sources = [
  'views/search/search_window.py',
  'views/search/search_result.py',
]

db_models_sources = [
	'db/models/project.py',
	'db/models/list.py',
	'db/models/task.py',
]

db_operations_sources = [
	'db/operations/project.py',
	'db/operations/list.py',
	'db/operations/task.py'
]


install_data('main.py', install_dir: moduledir)
install_data('views/window.py', install_dir: join_paths(moduledir, 'views'))
install_data(views_sidebar_sources, install_dir: join_paths(moduledir, 'views', 'sidebar'))
install_data(views_project_sources, install_dir: join_paths(moduledir, 'views', 'project'))
install_data(views_search_sources, install_dir: join_paths(moduledir, 'views', 'search'))
install_data('db/manager.py', install_dir: join_paths(moduledir, 'db'))
install_data(db_models_sources, install_dir: join_paths(moduledir, 'db', 'models'))
install_data(db_operations_sources, install_dir: join_paths(moduledir, 'db', 'operations'))
