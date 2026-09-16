Pod::Spec.new do |s|
  s.name           = 'ExpoPapermint'
  s.version        = '0.3.0'
  s.summary        = 'High-performance receipt and label compilation engine for Expo & React Native'
  s.description    = 'Native ESC/POS, StarPRNT, and thermal receipt compilation module powered by Papermint Rust engine'
  s.author         = 'Papermint Team'
  s.homepage       = 'https://papermint.zerathlabs.com'
  s.platforms      = {
    :ios => '16.4',
    :tvos => '16.4'
  }
  s.source         = { git: 'https://github.com/zerathlabs/papermint.git' }
  s.static_framework = true

  s.dependency 'ExpoModulesCore'

  # Swift/Objective-C compatibility
  s.pod_target_xcconfig = {
    'DEFINES_MODULE' => 'YES',
  }

  s.source_files = "**/*.{h,m,mm,swift,hpp,cpp}"

  if File.exist?(File.join(__dir__, 'PapermintMobile.xcframework'))
    s.vendored_frameworks = 'PapermintMobile.xcframework'
  elsif File.exist?(File.join(__dir__, 'libpapermint_mobile.a'))
    s.vendored_libraries = 'libpapermint_mobile.a'
  end
end
