pkgdatadir = join_paths(get_option('prefix'), get_option('datadir'), meson.project_name())
moduledir = join_paths(pkgdatadir, 'src')
gnome = import('gnome')

gnome.compile_resources('iplan',
  'iplan.gresource.xml',
  gresource_bundle: true,
  install: true,
  install_dir: pkgdatadir,
)

conf = configuration_data()
if get_option('profile') == 'development'
	conf.set_quoted('APPLICATION_ID', 'ir.imansalmani.IPlan.Devel')
else
	conf.set_quoted('APPLICATION_ID', 'ir.imansalmani.IPlan')
endif
conf.set_quoted('VERSION', meson.project_version())
conf.set_quoted('GETTEXT_PACKAGE', 'iplan')
conf.set_quoted('LOCALEDIR', join_paths(get_option('prefix'), get_option('localedir')))
conf.set_quoted('PKGDATADIR', pkgdatadir)

configure_file(
  input: 'config.rs.in',
  output: 'config.rs',
  configuration: conf,
)

# Copy the config.rs output to the source directory
run_command(
	'cp',
	join_paths(meson.project_build_root(), 'src', 'config.rs'),
	join_paths(meson.project_source_root(), 'src', 'config.rs'),
	check: true
)

# Run cargo fmt
run_command('cargo', 'fmt')

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
if get_option('profile') == 'default'
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
