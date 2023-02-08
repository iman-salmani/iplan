pkgdatadir = join_paths(get_option('prefix'), get_option('datadir'), meson.project_name())
moduledir = join_paths(pkgdatadir, 'src')
gnome = import('gnome')

gnome.compile_resources('iplan',
  'iplan.gresource.xml',
  gresource_bundle: true,
  install: true,
  install_dir: pkgdatadir,
)

# python = import('python')

conf = configuration_data()
conf.set_quoted('VERSION', meson.project_version())
conf.set_quoted('GETTEXT_PACKAGE', 'iplan')
conf.set_quoted('LOCALEDIR', join_paths(get_option('prefix'), get_option('localedir')))
conf.set_quoted('PKGDATADIR', pkgdatadir)
# conf.set('PYTHON', python.find_installation('python3').path())
# conf.set('VERSION', meson.project_version())
# conf.set('localedir', join_paths(get_option('prefix'), get_option('localedir')))
# conf.set('pkgdatadir', pkgdatadir)

configure_file(
  # input: 'iplan.in',
  # output: 'iplan',
  input: 'config.rs.in',
  output: 'config.rs',
  configuration: conf,
  # install: true,
  # install_dir: get_option('bindir')
)

# Copy the config.rs output to the source directory
run_command(
	'cp',
	join_paths(meson.project_build_root(), 'src', 'config.rs'),
	join_paths(meson.project_source_root(), 'src', 'config.rs'),
	check: true
)

# Rust options and env
cargo_bin = find_program('cargo')
cargo_opt = [ '--manifest-path', meson.project_source_root() / 'Cargo.toml' ]
cargo_opt += [ '--target-dir', meson.project_build_root() / 'src' ]
cargo_env = [ 'CARGO_HOME=' + meson.project_build_root() / 'cargo-home' ]

# Test
# cargo_test = custom_target(
# 	'cargo-test',
# 	build_by_default: true,
# 	build_always_stale: true,
# 	output: 'test.txt',
# 	console: true,
# 	command: [ cargo_bin, 'test', cargo_opt ],
# 	env: cargo_env
# )

# Check buildtype
if get_option('buildtype') == 'release'
	cargo_opt += [ '--release' ]
	rust_target = 'release'
else
	rust_target = 'debug'
endif

# Build
cargo_build = custom_target(
	'cargo-build',
	build_by_default: true,
	build_always_stale: true,
	output: meson.project_name(),
	console: true,
	install: true,
	install_dir: get_option('bindir'),
	command: [
		'env', cargo_env,
		cargo_bin, 'build',
		cargo_opt, '&&', 'cp', 'src' / rust_target / meson.project_name(), '@OUTPUT@',
	]
)

# views_sidebar_sources = [
#   'views/sidebar/sidebar.py',
#   'views/sidebar/sidebar_projects.py',
#   'views/sidebar/sidebar_project.py',
# ]
# views_project_sources = [
#   'views/project/project_header.py',
#   'views/project/project_lists.py',
#   'views/project/project_list.py',
#   'views/project/project_list_task.py',
#   'views/project/project_list_delete_dialog.py',
#   'views/project/project_edit_window.py',
#   'views/project/project_delete_dialog.py',
# ]

# views_search_sources = [
#   'views/search/search_window.py',
#   'views/search/search_result.py',
# ]

# db_models_sources = [
# 	'db/models/project.py',
# 	'db/models/list.py',
# 	'db/models/task.py',
# ]

# db_operations_sources = [
# 	'db/operations/project.py',
# 	'db/operations/list.py',
# 	'db/operations/task.py'
# ]


# install_data('main.py', install_dir: moduledir)
# install_data('views/window.py', install_dir: join_paths(moduledir, 'views'))
# install_data(views_sidebar_sources, install_dir: join_paths(moduledir, 'views', 'sidebar'))
# install_data(views_project_sources, install_dir: join_paths(moduledir, 'views', 'project'))
# install_data(views_search_sources, install_dir: join_paths(moduledir, 'views', 'search'))
# install_data('db/manager.py', install_dir: join_paths(moduledir, 'db'))
# install_data(db_models_sources, install_dir: join_paths(moduledir, 'db', 'models'))
# install_data(db_operations_sources, install_dir: join_paths(moduledir, 'db', 'operations'))
