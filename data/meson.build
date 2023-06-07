if get_option('profile') == 'development'
	application_id = 'ir.imansalmani.IPlan.Devel'
else
  application_id = 'ir.imansalmani.IPlan'
endif

desktop_file = i18n.merge_file(
        input: 'ir.imansalmani.IPlan.desktop.in',
       output: application_id + '.desktop',
         type: 'desktop',
       po_dir: '../po',
      install: true,
  install_dir: join_paths(get_option('datadir'), 'applications')
)

desktop_utils = find_program('desktop-file-validate', required: false)
if desktop_utils.found()
  test('Validate desktop file', desktop_utils, args: [desktop_file])
endif

appstream_file = i18n.merge_file(
        input: 'ir.imansalmani.IPlan.metainfo.xml.in',
       output: 'ir.imansalmani.IPlan.metainfo.xml',
       po_dir: '../po',
      install: true,
  install_dir: join_paths(get_option('datadir'), 'metainfo')
)

appstream_util = find_program('appstream-util', required: false)
if appstream_util.found()
  test('Validate appstream file', appstream_util, args: ['validate', appstream_file])
endif

install_data('ir.imansalmani.IPlan.gschema.xml',
  install_dir: join_paths(get_option('datadir'), 'glib-2.0/schemas')
)

compile_schemas = find_program('glib-compile-schemas', required: false)
if compile_schemas.found()
  test('Validate schema file',
       compile_schemas,
       args: ['--strict', '--dry-run', meson.current_source_dir()])
endif

service_conf = configuration_data()
service_conf.set('application_id', application_id)
service_conf.set('bindir', get_option('prefix') / get_option('bindir'))
configure_file(
  input: 'io.bassi.IPlan.service.in',
  output: '@0@.service'.format(application_id),
  configuration: service_conf,
  install: true,
  install_dir: get_option('datadir') / 'dbus-1/services',
)

subdir('icons')
