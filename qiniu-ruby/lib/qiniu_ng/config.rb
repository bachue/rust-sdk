# frozen_string_literal: true

module QiniuNg
  # 七牛客户端配置
  #
  # 提供客户端必要的配置信息
  class Config
    # @!visibility private
    DEFAULT_APPENDED_USER_AGENT = ["qiniu-ruby", VERSION, RUBY_ENGINE, RUBY_ENGINE_VERSION, RUBY_PLATFORM].freeze

    # 设置回调异常处理函数
    #
    # 由于回调函数内可能会抛出异常，而 SDK 使用的原生库无法接收到 Ruby 异常，
    # 因此，所有回调函数内抛出的异常将会在这个处理函数内进行处理。
    # 默认情况下，将会把错误详细信息输出到 STDERR。
    # 如果有需要，您可以通过赋值的方式定制处理方法。
    CallbackExceptionHandler = proc do |exception|
      STDERR.puts "Callback exception: #{exception.message}"
      exception.backtrace.each { |trace| STDERR.puts "\t#{trace}" }
    end

    # @!visibility private
    def initialize(config_ffi)
      @config = config_ffi
      @cache = {}
    end
    private_class_method :new

    # 创建客户端实例
    # @param [Boolean] use_https 是否使用 HTTPS 协议，默认为使用 HTTPS 协议
    # @param [String] api_host API 服务器地址（仅需要指定主机地址和端口，无需包含协议），默认将会使用七牛公有云的 API 服务器地址，仅在使用私有云时才需要配置
    # @param [String] rs_host RS 服务器地址（仅需要指定主机地址和端口，无需包含协议），默认将会使用七牛公有云的 RS 服务器地址，仅在使用私有云时才需要配置
    # @param [String] rsf_host RSF 服务器地址（仅需要指定主机地址和端口，无需包含协议），默认将会使用七牛公有云的 RSF 服务器地址，仅在使用私有云时才需要配置
    # @param [String] uc_host UC 服务器地址（仅需要指定主机地址和端口，无需包含协议），默认将会使用七牛公有云的 UC 服务器地址，仅在使用私有云时才需要配置
    # @param [String] uplog_host UpLog 服务器地址（仅需要指定主机地址和端口，无需包含协议），默认将会使用七牛公有云的 UpLog 服务器地址，仅在使用私有云时才需要配置
    # @param [Integer] batch_max_operation_size 最大批量操作数，默认为 1000
    # @param [Utils::Duration] http_connect_timeout HTTP 请求连接超时时长，默认为 5 秒
    # @param [Utils::Duration] http_request_timeout HTTP 请求超时时长，默认为 5 分钟
    # @param [Utils::Duration] tcp_keepalive_idle_timeout TCP KeepAlive 空闲时长，默认为 5 分钟
    # @param [Utils::Duration] tcp_keepalive_probe_interval TCP KeepAlive 探测包的发送间隔，默认为 5 秒
    # @param [Integer] http_low_transfer_speed HTTP 最低传输速度，与 http_low_transfer_speed_timeout 配合使用，单位为字节/秒，默认为 1024 字节/秒
    # @param [Utils::Duration] http_low_transfer_speed_timeout HTTP 最低传输速度维持时长，与 http_low_transfer_speed 配合使用，默认为 30 秒
    # @param [Integer] http_request_retries HTTP 请求重试次数，当 SDK 发送 HTTP 请求时发生错误，且该错误可以通过重试来解决时，SDK 将重试的次数。默认为 3 次
    # @param [Utils::Duration] http_request_retry_delay HTTP 请求重试前等待时间，当 SDK 发送 HTTP 请求时发生错误，且该错误可以通过重试来解决时，SDK 将等待一段时间并且重试，每次实际等待时长为该项值的 50% - 100% 之间的随机时长。默认为 1 秒，也就是说每次等待 500 毫秒至 1 秒间不等
    # @param [Integer] upload_block_size 上传分块尺寸，尺寸越小越适合弱网环境，必须是 4 MB 的倍数。单位为字节，默认为 4 MB
    # @param [Integer] upload_threshold 如果上传文件尺寸大于该值，将自动使用分片上传，否则，使用表单上传。单位为字节，默认为 4 MB
    # @param [Utils::Duration] upload_token_lifetime 上传凭证有效期，默认为 1 小时
    # @param [Boolean] upload_recorder_always_flush_records 设置进度记录文件始终刷新，默认不刷新
    # @param [String] upload_recorder_root_directory 设置上传进度记录仪文件根目录
    # @param [Utils::Duration] upload_recorder_upload_block_lifetime 设置文件分块有效期。对于超过有效期的分块，SDK 将重新上传，确保所有分块在创建文件时均有效，默认为 7 天，这是七牛公有云默认的配置。对于私有云的情况，需要参照私有云的配置来设置
    # @param [Symbol] uplog_file_lock_policy 设置文件锁策略，默认为 "在追加日志时为日志文件加共享锁，而上传时使用排他锁"
    # @param [Integer] uplog_file_max_size 设置上传日志文件的最大尺寸，单位为字节，默认为 4 MB
    # @param [String] uplog_file_path 设置上传日志文件路径
    # @param [Integer] uplog_file_upload_threshold 设置上传日志文件的上传阙值，单位为字节，默认为 4 KB
    # @return [Config] 返回创建的客户端配置实例
    def self.create(use_https: nil,
                    api_host: nil,
                    rs_host: nil,
                    rsf_host: nil,
                    uc_host: nil,
                    uplog_host: nil,
                    batch_max_operation_size: nil,
                    http_connect_timeout: nil,
                    http_low_transfer_speed: nil,
                    http_low_transfer_speed_timeout: nil,
                    http_request_retries: nil,
                    http_request_retry_delay: nil,
                    http_request_timeout: nil,
                    tcp_keepalive_idle_timeout: nil,
                    tcp_keepalive_probe_interval: nil,
                    upload_block_size: nil,
                    upload_threshold: nil,
                    upload_token_lifetime: nil,
                    upload_recorder_always_flush_records: nil,
                    upload_recorder_root_directory: nil,
                    upload_recorder_upload_block_lifetime: nil,
                    uplog_file_lock_policy: nil,
                    uplog_file_max_size: nil,
                    uplog_file_path: nil,
                    uplog_file_upload_threshold: nil)
      builder = Builder.new
      generate_default = true
      unless use_https.nil?
        builder.use_https = use_https
        generate_default = false
      end
      unless api_host.nil?
        builder.api_host = api_host
        generate_default = false
      end
      unless rs_host.nil?
        builder.rs_host = rs_host
        generate_default = false
      end
      unless rsf_host.nil?
        builder.rsf_host = rsf_host
        generate_default = false
      end
      unless uc_host.nil?
        builder.uc_host = uc_host
        generate_default = false
      end
      unless uplog_host.nil?
        builder.uplog_host = uplog_host
        generate_default = false
      end
      unless batch_max_operation_size.nil?
        builder.batch_max_operation_size = batch_max_operation_size
        generate_default = false
      end
      unless http_connect_timeout.nil?
        builder.http_connect_timeout = http_connect_timeout
        generate_default = false
      end
      unless http_low_transfer_speed.nil?
        builder.http_low_transfer_speed = http_low_transfer_speed
        generate_default = false
      end
      unless http_low_transfer_speed_timeout.nil?
        builder.http_low_transfer_speed_timeout = http_low_transfer_speed_timeout
        generate_default = false
      end
      unless http_request_retries.nil?
        builder.http_request_retries = http_request_retries
        generate_default = false
      end
      unless http_request_retry_delay.nil?
        builder.http_request_retry_delay = http_request_retry_delay
        generate_default = false
      end
      unless http_request_timeout.nil?
        builder.http_request_timeout = http_request_timeout
        generate_default = false
      end
      unless tcp_keepalive_idle_timeout.nil?
        builder.tcp_keepalive_idle_timeout = tcp_keepalive_idle_timeout
        generate_default = false
      end
      unless tcp_keepalive_probe_interval.nil?
        builder.tcp_keepalive_probe_interval = tcp_keepalive_probe_interval
        generate_default = false
      end
      unless upload_block_size.nil?
        builder.upload_block_size = upload_block_size
        generate_default = false
      end
      unless upload_threshold.nil?
        builder.upload_threshold = upload_threshold
        generate_default = false
      end
      unless upload_token_lifetime.nil?
        builder.upload_token_lifetime = upload_token_lifetime
        generate_default = false
      end
      unless upload_recorder_always_flush_records.nil?
        builder.upload_recorder_always_flush_records = upload_recorder_always_flush_records
        generate_default = false
      end
      unless upload_recorder_root_directory.nil?
        builder.upload_recorder_root_directory = upload_recorder_root_directory
        generate_default = false
      end
      unless upload_recorder_upload_block_lifetime.nil?
        builder.upload_recorder_upload_block_lifetime = upload_recorder_upload_block_lifetime
        generate_default = false
      end
      unless uplog_file_lock_policy.nil?
        builder.uplog_file_lock_policy = uplog_file_lock_policy
        generate_default = false
      end
      unless uplog_file_max_size.nil?
        builder.uplog_file_max_size = uplog_file_max_size
        generate_default = false
      end
      unless uplog_file_path.nil?
        builder.uplog_file_path = uplog_file_path
        generate_default = false
      end
      unless uplog_file_upload_threshold.nil?
        builder.uplog_file_upload_threshold = uplog_file_upload_threshold
        generate_default = false
      end
      config_ffi = Error.wrap_ffi_function do
                     if generate_default
                       Bindings::Config.new_default
                     else
                       Bindings::Config.build(builder.instance_variable_get(:@builder))
                     end
                   end
      new(config_ffi)
    end

    # @!visibility private
    def inspect
      "#<#{self.class.name}>"
    end

    # @!method use_https?
    #   是否使用 HTTPS 协议
    #   @return [Bool] 是否使用 HTTPS 协议
    # @!method domains_manager_auto_persistent_disabled?
    #   域名管理器是否禁用自动持久化
    #   @return [Bool] 域名管理器是否禁用自动持久化
    # @!method domains_manager_url_resolution_disabled?
    #   域名管理器是否禁用 URL 域名预解析
    #   @return [Bool] 域名管理器是否禁用 URL 域名预解析
    # @!method upload_recorder_always_flush_records?
    #   进度记录文件是否始终刷新
    #   @return [Bool] 进度记录文件是否始终刷新
    # @!method uplog_enabled?
    #   是否启用上传日志记录仪
    #   @return [Bool] 是否启用上传日志记录仪

    # 设置布尔值属性 Getters
    %i[use_https
       domains_manager_auto_persistent_disabled
       domains_manager_url_resolution_disabled
       upload_recorder_always_flush_records
       uplog_enabled].each do |method|
      define_method(:"#{method}?") do
        @config.public_send(:"get_#{method}")
      end
    end

    # @!method batch_max_operation_size
    #   最大批量操作数
    #   @return [Integer] 最大批量操作数
    # @!method domains_manager_url_resolve_retries
    #   域名管理器的 URL 域名预解析重试次数
    #   @return [Integer] 域名管理器的 URL 域名预解析重试次数
    # @!method http_request_retries
    #   HTTP 请求重试次数
    #   @return [Integer] HTTP 请求重试次数
    # @!method http_low_transfer_speed
    #   HTTP 最低传输速度维持时长
    #   @return [Integer] HTTP 最低传输速度维持时长
    # @!method upload_block_size
    #   上传分块尺寸
    #   @return [Integer] 上传分块尺寸，单位为字节
    # @!method upload_threshold
    #   分片上传策略阙值
    #   @return [Integer] 分片上传策略阙值，单位为字节

    # 设置整型属性 Getters
    %i[batch_max_operation_size
       domains_manager_url_resolve_retries
       http_request_retries
       http_low_transfer_speed
       upload_block_size
       upload_threshold].each do |method|
      define_method(method) do
        @config.public_send(:"get_#{method}")
      end
    end

    # @!method api_host
    #   API 服务器地址
    #   @return [String] API 服务器地址
    # @!method api_url
    #   API 服务器 URL
    #   @return [String] API 服务器 URL
    # @!method rs_host
    #   RS 服务器地址
    #   @return [String] RS 服务器地址
    # @!method rs_url
    #   RS 服务器 URL
    #   @return [String] RS 服务器 URL
    # @!method rsf_host
    #   RSF 服务器地址
    #   @return [String] RSF 服务器地址
    # @!method rsf_url
    #   RSF 服务器 URL
    #   @return [String] RSF 服务器 URL
    # @!method uc_host
    #   UC 服务器地址
    #   @return [String] UC 服务器地址
    # @!method uc_url
    #   UC 服务器 URL
    #   @return [String] UC 服务器 URL
    # @!method uplog_host
    #   UpLog 服务器地址
    #   @return [String] UpLog 服务器地址
    # @!method uplog_url
    #   UpLog 服务器 URL
    #   @return [String] UpLog 服务器 URL
    # @!method user_agent
    #   用户代理
    #   @return [String] 用户代理
    # @!method upload_recorder_root_directory
    #   上传进度记录仪文件根目录
    #   @return [String] 上传进度记录仪文件根目录
    # @!method uplog_file_path
    #   上传日志文件路径
    #   @return [String] 上传日志文件路径

    # 设置字符串属性 Getters
    %i[api_host
       api_url
       rs_host
       rs_url
       rsf_host
       rsf_url
       uc_host
       uc_url
       uplog_host
       uplog_url
       user_agent
       upload_recorder_root_directory
       uplog_file_path].each do |method|
      define_method(method) do
        @cache[method] ||= @config.public_send(:"get_#{method}")
        return nil if @cache[method].is_null
        @cache[method].get_cstr
      end
    end

    # @!method domains_manager_auto_persistent_interval
    #   域名管理器的自动持久化间隔时间
    #   @return [Utils::Duration] 域名管理器的自动持久化间隔时间
    # @!method domains_manager_resolutions_cache_lifetime
    #   域名管理器的域名解析缓存生命周期
    #   @return [Utils::Duration] 域名管理器的域名解析缓存生命周期
    # @!method domains_manager_url_frozen_duration
    #   域名管理器的 URL 冻结时长
    #   @return [Utils::Duration] 域名管理器的 URL 冻结时长
    # @!method domains_manager_url_resolve_retry_delay
    #   域名管理器的 URL 域名预解析重试前等待时间
    #   @return [Utils::Duration] 域名管理器的 URL 域名预解析重试前等待时间
    # @!method http_connect_timeout
    #   HTTP 请求连接超时时长
    #   @return [Utils::Duration] HTTP 请求连接超时时长
    # @!method http_low_transfer_speed_timeout
    #   HTTP 最低传输速度
    #   @return [Utils::Duration] HTTP 最低传输速度
    # @!method http_request_retry_delay
    #   HTTP 请求重试前等待时间
    #   @return [Utils::Duration] HTTP 请求重试前等待时间
    # @!method http_request_timeout
    #   HTTP 请求超时时长
    #   @return [Utils::Duration] HTTP 请求超时时长
    # @!method tcp_keepalive_idle_timeout
    #   TCP KeepAlive 空闲时长
    #   @return [Utils::Duration] TCP KeepAlive 空闲时长
    # @!method tcp_keepalive_probe_interval
    #   TCP KeepAlive 探测包的发送间隔
    #   @return [Utils::Duration] TCP KeepAlive 探测包的发送间隔
    # @!method upload_recorder_upload_block_lifetime
    #   文件分块有效期
    #   @return [Utils::Duration] 文件分块有效期
    # @!method upload_token_lifetime
    #   上传凭证有效期
    #   @return [Utils::Duration] 上传凭证有效期

    # 设置时间型属性 Getters
    %i[domains_manager_auto_persistent_interval
       domains_manager_resolutions_cache_lifetime
       domains_manager_url_frozen_duration
       domains_manager_url_resolve_retry_delay
       http_connect_timeout
       http_low_transfer_speed_timeout
       http_request_retry_delay
       http_request_timeout
       tcp_keepalive_idle_timeout
       tcp_keepalive_probe_interval
       upload_recorder_upload_block_lifetime
       upload_token_lifetime].each do |method|
      define_method(method) do
        Utils::Duration.new(seconds: @config.public_send(:"get_#{method}"))
      end
    end

    # 上传日志文件锁策略
    # @return [Symbol] 上传日志文件锁策略
    def uplog_file_lock_policy
      policy = Bindings::CoreFFI::QiniuNgUploadLoggerLockPolicyTWrapper.new
      return nil unless @config.get_uplog_file_lock_policy(policy)
      case policy[:inner]
      when :qiniu_ng_lock_policy_lock_shared_duration_appending_and_lock_exclusive_duration_uploading
        :lock_shared_duration_appending_and_lock_exclusive_duration_uploading
      when :qiniu_ng_lock_policy_always_lock_exclusive
        :always_lock_exclusive
      when :qiniu_ng_lock_policy_none
        :none
      else
        raise RuntimeError, "unrecognized lock policy: #{policy[:enum].inspect}"
      end
    end

    # 上传日志文件的上传阙值
    # @return [Symbol] 上传日志文件的上传阙值，单位为字节
    def uplog_file_upload_threshold
      u32 = Bindings::CoreFFI::U32.new
      return nil unless @config.get_uplog_file_upload_threshold(u32)
      u32[:value]
    end

    # 上传日志文件的最大尺寸
    # @return [Symbol] 上传日志文件的最大尺寸，单位为字节
    def uplog_file_max_size
      u32 = Bindings::CoreFFI::U32.new
      return nil unless @config.get_uplog_file_max_size(u32)
      u32[:value]
    end

    # 七牛客户端配置生成器
    #
    # 通过多次调用方法修改配置数据，将具有比 {Config#initialize} 更强大的功能
    #
    # @example
    #   config = QiniuNg::Config::Builder.new.use_https(true).build!
    class Builder
      # 创建默认的七牛客户端配置生成器
      def initialize
        @builder = self.class.send(:new_default)
      end

      # 生成客户端配置
      # @return [Config] 返回新建的客户端配置实例
      def build!
        config_ffi = Error.wrap_ffi_function do
                       Bindings::Config.build(@builder)
                     end
        Config.send(:new, config_ffi)
      ensure
        @builder = self.class.send(:new_default)
      end

      # @!visibility private
      def self.new_default
        Bindings::ConfigBuilder.new!.tap do |builder|
          builder.set_appended_user_agent(DEFAULT_APPENDED_USER_AGENT.join('/'))
          builder.set_http_request_final_handler(HTTPRequestFinalHandler, nil)
        end
      end
      private_class_method :new_default

      # @!method enable_uplog
      #   启用上传日志记录仪，默认为启用
      #   @return [Builder] 返回自身，可以形成链式调用
      # @!method disable_uplog
      #   禁用上传日志记录仪，默认为启用
      #   @return [Builder] 返回自身，可以形成链式调用
      # @!method domains_manager_disable_auto_persistent
      #   禁止域名管理器自动持久化，默认为启用
      #   @return [Builder] 返回自身，可以形成链式调用
      # @!method domains_manager_disable_url_resolution
      #   禁止域名管理器 URL 域名预解析，默认为启用
      #   @return [Builder] 返回自身，可以形成链式调用
      # @!method domains_manager_enable_url_resolution
      #   启用域名管理器 URL 域名预解析，默认为启用
      #   @return [Builder] 返回自身，可以形成链式调用

      # 设置无参数 Setters
      %i[enable_uplog
         disable_uplog
         domains_manager_disable_auto_persistent
         domains_manager_disable_url_resolution
         domains_manager_enable_url_resolution].each do |method|
        define_method(method) do
          @builder.public_send(method)
          self
        end
      end

      # @!method use_https(use_https)
      #   是否使用 HTTPS 协议
      #   @param [Boolean] use_https 是否使用 HTTPS 协议，默认为使用 HTTPS 协议
      #   @return [Builder] 返回自身，可以形成链式调用
      # @!method upload_recorder_always_flush_records(always_flush_records)
      #   设置进度记录文件始终刷新
      #   @param [Boolean] always_flush_records 进度记录文件是否始终刷新
      #   @return [Builder] 返回自身，可以形成链式调用

      # 设置布尔型参数 Setters
      %i[use_https
         upload_recorder_always_flush_records].each do |method|
        define_method(method) do |arg|
          @builder.public_send(method, !!arg)
          self
        end
        alias_method :"#{method}=", method
      end

      # 设置文件锁策略
      #
      # 为了防止上传文件的过程中，上传日志文件被多个进程同时修改引发竞争，因此需要在操作日志文件时使用文件锁保护。
      # 默认策略 :lock_shared_duration_appending_and_lock_exclusive_duration_uploading 为在追加日志时为日志文件加共享锁，而上传时使用排他锁，尽可能做到安全和性能之间的平衡。
      #
      # 但在有些场景下中，并发追加日志文件同样会引发竞争，此时需要改用 :always_lock_exclusive 策略。
      # 此外，如果确定当前操作系统内不会有多个进程同时上传文件，或不同进程不会使用相同路径的日志时，
      # 也可以使用 :none 策略，减少文件锁的性能影响。
      #
      # @param [Symbol] lock_policy 上传日志文件锁策略
      # @return [Builder] 返回自身，可以形成链式调用
      def uplog_file_lock_policy(lock_policy)
        lock_policy = case lock_policy.to_sym
                      when :lock_shared_duration_appending_and_lock_exclusive_duration_uploading
                        :qiniu_ng_lock_policy_lock_shared_duration_appending_and_lock_exclusive_duration_uploading
                      when :always_lock_exclusive
                        :qiniu_ng_lock_policy_always_lock_exclusive
                      when :none
                        :qiniu_ng_lock_policy_none
                      else
                        raise ArgumentError, "invalid lock policy: #{lock_policy.inspect}"
                      end
        @builder.uplog_file_lock_policy(lock_policy)
        self
      end
      alias uplog_file_lock_policy= uplog_file_lock_policy

      # 创建一个新的域名管理器
      # @param [String] persistent_file 新的域名管理器的持久化路径，如果传入 nil 则表示禁止持久化
      # @return [Builder] 返回自身，可以形成链式调用
      def create_new_domains_manager(persistent_file = nil)
        Error.wrap_ffi_function do
          @builder.create_new_domains_manager(persistent_file&.to_s)
        end
        self
      end

      # 从指定路径加载域名管理器
      # @param [String] persistent_file 持久化路径
      # @return [Builder] 返回自身，可以形成链式调用
      def load_domains_manager_from_file(persistent_file)
        Error.wrap_ffi_function do
          @builder.create_new_domains_manager(persistent_file.to_s)
        end
        self
      end

      # 设置追加用户代理
      #
      # SDK 本身会包含预定的用户代理字符串，您不能修改该字符串，但可以向该字符串追加更多内容
      #
      # @param [String] user_agent 追加的用户代理
      # @return [Builder] 返回自身，可以形成链式调用
      def set_appended_user_agent(user_agent)
        user_agent = [user_agent.to_s] unless user_agent.is_a?(Array)
        user_agent = (DEFAULT_APPENDED_USER_AGENT + user_agent).join('/')
        @builder.set_appended_user_agent(user_agent)
        self
      end

      # 追加 HTTP 请求响应后回调函数
      #
      # 您可以利用该特性输出 HTTP 日志或对 HTTP 响应内容进行修改。
      # 但注意，您必须确保不破坏响应中必要的内容，否则 SDK 可能无法处理该响应。
      #
      # 如果在回调中发生错误，您可以抛出对应的异常。但只有以下几种异常会被 SDK 处理:
      #
      #   - `Error::IOHandlerError` - IO 异常
      #
      #   - `Error::OSHandlerError` - 系统调用异常
      #
      #   - `Error::UnexpectedRedirectHandlerError` - 非预期的重定向错误
      #
      #   - `Error::UserCancelledHandlerError` - 用户取消异常
      #
      #   - `Error::JSONHandlerError` - JSON 错误
      #
      #   - `Error::ResponseStatusCodeHandlerError` - 响应状态码错误
      #
      #   - `Error::CurlHandlerError` - Curl 错误
      #
      # @yield [request, response] 回调函数
      # @yieldparam [HTTP::Request] request HTTP 请求
      # @yieldparam [HTTP::Response] response HTTP 响应
      # @return [Builder] 返回自身，可以形成链式调用
      def append_http_request_after_action_handler(&handler)
        idx = CallbackData.put(handler)
        @builder.append_http_request_after_action_handler(HTTPRequestAfterActionHandler, idx)
        self
      end

      # 追加 HTTP 请求前回调函数
      #
      # 您可以利用该特性输出 HTTP 日志或对 HTTP 请求内容进行修改。
      # 但注意，您必须确保不破坏请求中必要的内容，否则七牛服务器可能无法处理该请求。
      #
      # 如果在回调中发生错误，您可以抛出对应的异常。但只有以下几种异常会被 SDK 处理:
      #
      #   - `Error::IOHandlerError` - IO 异常
      #
      #   - `Error::OSHandlerError` - 系统调用异常
      #
      #   - `Error::UnexpectedRedirectHandlerError` - 非预期的重定向错误
      #
      #   - `Error::UserCancelledHandlerError` - 用户取消异常
      #
      #   - `Error::JSONHandlerError` - JSON 错误
      #
      #   - `Error::ResponseStatusCodeHandlerError` - 响应状态码错误
      #
      #   - `Error::CurlHandlerError` - Curl 错误
      #
      # @yield [request] 回调函数
      # @yieldparam [HTTP::Request] request HTTP 请求
      # @return [Builder] 返回自身，可以形成链式调用
      def append_http_request_before_action_handler(&handler)
        idx = CallbackData.put(handler)
        @builder.append_http_request_before_action_handler(HTTPRequestBeforeActionHandler, idx)
        self
      end

      # 新增 HTTP 请求响应后回调函数
      #
      # 您可以利用该特性输出 HTTP 日志或对 HTTP 响应内容进行修改。
      # 但注意，您必须确保不破坏响应中必要的内容，否则 SDK 可能无法处理该响应。
      #
      # 如果在回调中发生错误，您可以抛出对应的异常。但只有以下几种异常会被 SDK 处理:
      #
      #   - `Error::IOHandlerError` - IO 异常
      #
      #   - `Error::OSHandlerError` - 系统调用异常
      #
      #   - `Error::UnexpectedRedirectHandlerError` - 非预期的重定向错误
      #
      #   - `Error::UserCancelledHandlerError` - 用户取消异常
      #
      #   - `Error::JSONHandlerError` - JSON 错误
      #
      #   - `Error::ResponseStatusCodeHandlerError` - 响应状态码错误
      #
      #   - `Error::CurlHandlerError` - Curl 错误
      #
      # @yield [request, response] 回调函数
      # @yieldparam [HTTP::Request] request HTTP 请求
      # @yieldparam [HTTP::Response] response HTTP 响应
      # @return [Builder] 返回自身，可以形成链式调用
      def prepend_http_request_after_action_handler(&handler)
        idx = CallbackData.put(handler)
        @builder.prepend_http_request_after_action_handler(HTTPRequestAfterActionHandler, idx)
        self
      end

      # 新增 HTTP 请求前回调函数
      #
      # 您可以利用该特性输出 HTTP 日志或对 HTTP 请求内容进行修改。
      # 但注意，您必须确保不破坏请求中必要的内容，否则七牛服务器可能无法处理该请求。
      #
      # 如果在回调中发生错误，您可以抛出对应的异常。但只有以下几种异常会被 SDK 处理:
      #
      #   - `Error::IOHandlerError` - IO 异常
      #
      #   - `Error::OSHandlerError` - 系统调用异常
      #
      #   - `Error::UnexpectedRedirectHandlerError` - 非预期的重定向错误
      #
      #   - `Error::UserCancelledHandlerError` - 用户取消异常
      #
      #   - `Error::JSONHandlerError` - JSON 错误
      #
      #   - `Error::ResponseStatusCodeHandlerError` - 响应状态码错误
      #
      #   - `Error::CurlHandlerError` - Curl 错误
      #
      # @yield [request] 回调函数
      # @yieldparam [HTTP::Request] request HTTP 请求
      # @return [Builder] 返回自身，可以形成链式调用
      def prepend_http_request_before_action_handler(&handler)
        idx = CallbackData.put(handler)
        @builder.prepend_http_request_before_action_handler(HTTPRequestBeforeActionHandler, idx)
        self
      end

      # 设置 HTTP 请求处理函数
      #
      # 如果在处理函数中发生错误，您可以抛出对应的异常。但只有以下几种异常会被 SDK 处理:
      #
      #   - `Error::IOHandlerError` - IO 异常
      #
      #   - `Error::OSHandlerError` - 系统调用异常
      #
      #   - `Error::UnexpectedRedirectHandlerError` - 非预期的重定向错误
      #
      #   - `Error::UserCancelledHandlerError` - 用户取消异常
      #
      #   - `Error::JSONHandlerError` - JSON 错误
      #
      #   - `Error::ResponseStatusCodeHandlerError` - 响应状态码错误
      #
      #   - `Error::CurlHandlerError` - Curl 错误
      #
      # @yield [request, response] 处理函数
      # @yieldparam [HTTP::Request] request HTTP 请求
      # @yieldparam [HTTP::Response] response HTTP 响应
      # @return [Builder] 返回自身，可以形成链式调用
      def http_request_handler(&handler)
        idx = CallbackData.put(handler)
        @builder.set_http_call_handler(HTTPRequestAfterActionHandler, idx)
        self
      end

      # @!method api_host(api_host)
      #   设置 API 服务器地址
      #   @param [Boolean] api_host API 服务器地址（仅需要指定主机地址和端口，无需包含协议）
      #   @return [Builder] 返回自身，可以形成链式调用
      # @!method rs_host(rs_host)
      #   设置 RS 服务器地址
      #   @param [Boolean] rs_host RS 服务器地址（仅需要指定主机地址和端口，无需包含协议）
      #   @return [Builder] 返回自身，可以形成链式调用
      # @!method rsf_host(rsf_host)
      #   设置 RSF 服务器地址
      #   @param [Boolean] rsf_host RSF 服务器地址（仅需要指定主机地址和端口，无需包含协议）
      #   @return [Builder] 返回自身，可以形成链式调用
      # @!method uc_host(uc_host)
      #   设置 UC 服务器地址
      #   @param [Boolean] uc_host UC 服务器地址（仅需要指定主机地址和端口，无需包含协议）
      #   @return [Builder] 返回自身，可以形成链式调用
      # @!method uplog_host(uplog_host)
      #   设置 UpLog 服务器地址
      #   @param [Boolean] uplog_host UpLog 服务器地址（仅需要指定主机地址和端口，无需包含协议）
      #   @return [Builder] 返回自身，可以形成链式调用
      # @!method domains_manager_persistent_file_path(persistent_file_path)
      #   设置域名管理器的持久化路径
      #   @param [Boolean] persistent_file_path 持久化路径，如果传入 nil 则表示禁止持久化
      #   @return [Builder] 返回自身，可以形成链式调用
      # @!method domains_manager_pre_resolve_url(pre_resolve_url)
      #   添加域名预解析 URL
      #
      #   当客户端配置生成器生成前，可以指定多个预解析 URL 域名。
      #   而生成时，将以异步的方式预解析 URL 域名，并将结果缓存在域名管理器内
      #
      #   @param [Boolean] pre_resolve_url 域名预解析 URL
      #   @return [Builder] 返回自身，可以形成链式调用
      # @!method upload_recorder_root_directory(root_directory)
      #   设置上传进度记录仪文件根目录
      #
      #   默认的文件系统记录仪目录规则如下：
      #     1. 尝试在操作系统特定的缓存目录下创建 qiniu_sdk/records 目录。
      #     2. 如果成功，则使用 qiniu_sdk/records 目录。
      #     3. 如果失败，则直接使用临时目录。
      #
      #   @param [Boolean] root_directory 文件根目录
      #   @return [Builder] 返回自身，可以形成链式调用
      # @!method uplog_file_path(path)
      #   设置上传日志文件路径
      #
      #   默认的上传日志文件路径规则如下：
      #     1. 尝试在操作系统特定的缓存目录下创建 qiniu_sdk 目录。
      #     2. 如果成功，则使用 qiniu_sdk 目录下的 upload.log。
      #     3. 如果失败，则直接使用临时目录下的 upload.log。
      #
      #   @param [Boolean] path 上传日志文件路径
      #   @return [Builder] 返回自身，可以形成链式调用

      %i[api_host
         rs_host
         rsf_host
         uc_host
         uplog_host
         domains_manager_persistent_file_path
         domains_manager_pre_resolve_url
         upload_recorder_root_directory
         uplog_file_path].each do |method|
        define_method(method) do |arg|
          Error.wrap_ffi_function do
            @builder.public_send(method, arg.to_s)
          end
          self
        end
        alias_method :"#{method}=", method
      end

      # @!method batch_max_operation_size(size)
      #   设置最大批量操作数
      #
      #   默认为 1000
      #
      #   @param [Integer] size 最大批量操作数
      #   @return [Builder] 返回自身，可以形成链式调用
      #   @raise [RangeError] 超过最大范围
      # @!method domains_manager_url_resolve_retries(retries)
      #   设置域名管理器的 URL 域名预解析重试次数
      #
      #   默认为 10 次
      #
      #   @param [Integer] retries 重试次数
      #   @return [Builder] 返回自身，可以形成链式调用
      #   @raise [RangeError] 超过最大范围
      # @!method http_low_transfer_speed(speed)
      #   设置 HTTP 最低传输速度
      #
      #   与 http_low_transfer_speed_timeout 配合使用。
      #   当 HTTP 传输速度低于最低传输速度 http_low_transfer_speed 并维持超过 http_low_transfer_speed_timeout 的时长，则出错。
      #   SDK 会自动重试，或出错退出
      #
      #   默认为 1024 字节/秒
      #
      #   @param [Integer] speed 最低传输速度，单位为字节/秒
      #   @return [Builder] 返回自身，可以形成链式调用
      #   @raise [RangeError] 超过最大范围
      # @!method http_request_retries(retries)
      #   设置 HTTP 请求重试次数
      #
      #   当 SDK 发送 HTTP 请求时发生错误，且该错误可以通过重试来解决时，SDK 将重试的次数
      #
      #   默认为 3 次
      #
      #   @param [Integer] retries 重试次数
      #   @return [Builder] 返回自身，可以形成链式调用
      #   @raise [RangeError] 超过最大范围
      # @!method upload_block_size(size)
      #   设置上传分块尺寸
      #
      #   默认为 4 MB，尺寸越小越适合弱网环境，但必须是 4 MB 的倍数
      #
      #   @param [Integer] size 上传分块尺寸，单位为字节
      #   @return [Builder] 返回自身，可以形成链式调用
      #   @raise [RangeError] 超过最大范围
      # @!method upload_threshold(threshold)
      #   设置分片上传策略阙值
      #
      #   如果上传文件尺寸大于该值，将自动使用分片上传，否则，使用表单上传
      #
      #   默认为 4 MB
      #
      #   @param [Integer] threshold 分片上传策略阙值，单位为字节
      #   @return [Builder] 返回自身，可以形成链式调用
      #   @raise [RangeError] 超过最大范围
      # @!method uplog_file_max_size(max_size)
      #   设置上传日志文件的最大尺寸
      #
      #   当上传日志文件尺寸大于指定尺寸时，将不会再记录任何数据到日志内。
      #   防止在上传发生困难时日志文件无限制膨胀。
      #
      #   该值必须大于 upload_threshold，默认为 4 MB
      #
      #   @param [Integer] max_size 上传日志文件的最大尺寸，单位为字节
      #   @return [Builder] 返回自身，可以形成链式调用
      #   @raise [RangeError] 超过最大范围
      # @!method uplog_file_upload_threshold(threshold)
      #   设置上传日志文件的上传阙值
      #
      #   当且仅当上传日志文件尺寸大于阙值时才会上传日志
      #
      #   默认为 4 KB
      #
      #   @param [Integer] threshold 上传阙值，单位为字节
      #   @return [Builder] 返回自身，可以形成链式调用
      #   @raise [RangeError] 超过最大范围
      # @!method domains_manager_auto_persistent_interval(interval)
      #   设置域名管理器的自动持久化间隔时间
      #
      #   当自动持久化被启用，且存在持久化路径时，域名管理器将定期自动保存自身状态
      #
      #   默认间隔时间为三十分钟
      #
      #   @param [Utils::Duration] interval 自动持久化间隔时间
      #   @return [Builder] 返回自身，可以形成链式调用
      #   @raise [RangeError] 超过最大范围
      # @!method domains_manager_resolutions_cache_lifetime(lifetime)
      #   设置域名管理器的域名解析缓存生命周期
      #
      #   默认缓存一小时
      #
      #   @param [Utils::Duration] lifetime 域名解析缓存生命周期
      #   @return [Builder] 返回自身，可以形成链式调用
      #   @raise [RangeError] 超过最大范围
      # @!method domains_manager_url_frozen_duration(url_frozen_duration)
      #   设置域名管理器的 URL 冻结时长
      #
      #   当 SDK 发送 HTTP 请求时，如果发现网络或服务异常，靠重试无法解决的，则冻结所访问的服务器 URL。
      #   被冻结的服务器在冻结期间将无法被访问
      #
      #   默认冻结十分钟
      #
      #   @param [Utils::Duration] url_frozen_duration URL 冻结时长
      #   @return [Builder] 返回自身，可以形成链式调用
      #   @raise [RangeError] 超过最大范围
      # @!method domains_manager_url_resolve_retry_delay(delay)
      #   设置域名管理器的 URL 域名预解析重试前等待时间
      #
      #   当 SDK 预解析域名时发送错误时，SDK 将等待一段时间并且重试。
      #   每次实际等待时长为该项值的 50% - 100% 之间的随机时长。
      #
      #   默认为 1 秒，也就是说每次等待 500 毫秒至 1 秒间不等
      #
      #   @param [Utils::Duration] delay 等待时间
      #   @return [Builder] 返回自身，可以形成链式调用
      #   @raise [RangeError] 超过最大范围
      # @!method http_connect_timeout(timeout)
      #   设置 HTTP 请求连接超时时长
      #
      #   默认为 5 秒
      #
      #   @param [Utils::Duration] timeout 超时时长
      #   @return [Builder] 返回自身，可以形成链式调用
      #   @raise [RangeError] 超过最大范围
      # @!method http_low_transfer_speed_timeout(timeout)
      #   设置 HTTP 最低传输速度维持时长
      #
      #   与 http_low_transfer_speed 配合使用。
      #   当 HTTP 传输速度低于最低传输速度 http_low_transfer_speed 并维持超过 http_low_transfer_speed_timeout 的时长，则出错。
      #   SDK 会自动重试，或出错退出
      #
      #   默认为 30 秒
      #
      #   @param [Utils::Duration] timeout 最低传输速度维持时长
      #   @return [Builder] 返回自身，可以形成链式调用
      #   @raise [RangeError] 超过最大范围
      # @!method http_request_retry_delay(delay)
      #   设置 HTTP 请求重试前等待时间
      #
      #   当 SDK 发送 HTTP 请求时发生错误，且该错误可以通过重试来解决时，SDK 将等待一段时间并且重试
      #   每次实际等待时长为该项值的 50% - 100% 之间的随机时长
      #
      #   默认为 1 秒，也就是说每次等待 500 毫秒至 1 秒间不等
      #
      #   @param [Utils::Duration] delay 等待时间
      #   @return [Builder] 返回自身，可以形成链式调用
      #   @raise [RangeError] 超过最大范围
      # @!method http_request_timeout(timeout)
      #   设置 HTTP 请求超时时长
      #
      #   默认为 5 分钟
      #
      #   @param [Utils::Duration] timeout 超时时长
      #   @return [Builder] 返回自身，可以形成链式调用
      #   @raise [RangeError] 超过最大范围
      # @!method tcp_keepalive_idle_timeout(timeout)
      #   设置 TCP KeepAlive 空闲时长
      #
      #   默认为 5 分钟
      #
      #   @param [Utils::Duration] timeout 空闲时长
      #   @return [Builder] 返回自身，可以形成链式调用
      #   @raise [RangeError] 超过最大范围
      # @!method tcp_keepalive_probe_interval(interval)
      #   设置 TCP KeepAlive 探测包的发送间隔
      #
      #   默认为 5 秒
      #
      #   @param [Utils::Duration] interval 发送间隔
      #   @return [Builder] 返回自身，可以形成链式调用
      #   @raise [RangeError] 超过最大范围
      # @!method tcp_keepalive_probe_interval(interval)
      #   设置 TCP KeepAlive 探测包的发送间隔
      #
      #   默认为 5 秒
      #
      #   @param [Utils::Duration] interval 发送间隔
      #   @return [Builder] 返回自身，可以形成链式调用
      #   @raise [RangeError] 超过最大范围
      # @!method upload_recorder_upload_block_lifetime(lifetime)
      #   设置文件分块有效期
      #
      #   对于超过有效期的分块，SDK 将重新上传，确保所有分块在创建文件时均有效
      #
      #   默认为 7 天，这是七牛公有云默认的配置。对于私有云的情况，需要参照私有云的配置来设置
      #
      #   @param [Utils::Duration] lifetime 文件分块有效期
      #   @return [Builder] 返回自身，可以形成链式调用
      #   @raise [RangeError] 超过最大范围
      # @!method upload_token_lifetime(lifetime)
      #   设置上传凭证有效期
      #
      #   默认为 1 小时
      #
      #   @param [Utils::Duration] lifetime 上传凭证有效期
      #   @return [Builder] 返回自身，可以形成链式调用
      #   @raise [RangeError] 超过最大范围

      # 设置整型和时间型属性 Setters
      [[:batch_max_operation_size, 0, 1 << 32 - 1, false],
       [:domains_manager_auto_persistent_interval, 0, 1 << 64 - 1, true],
       [:domains_manager_resolutions_cache_lifetime, 0, 1 << 64 - 1, true],
       [:domains_manager_url_frozen_duration, 0, 1 << 64 - 1, true],
       [:domains_manager_url_resolve_retries, 0, 1 << 32 - 1, false],
       [:domains_manager_url_resolve_retry_delay, 0, 1 << 64 - 1, true],
       [:http_connect_timeout, 0, 1 << 64 - 1, true],
       [:http_low_transfer_speed, 0, 1 << 32 - 1, false],
       [:http_low_transfer_speed_timeout, 0, 1 << 64 - 1, true],
       [:http_request_retries, 0, 1 << 32 - 1, false],
       [:http_request_retry_delay, 0, 1 << 64 - 1, true],
       [:http_request_timeout, 0, 1 << 64 - 1, true],
       [:tcp_keepalive_idle_timeout, 0, 1 << 64 - 1, true],
       [:tcp_keepalive_probe_interval, 0, 1 << 64 - 1, true],
       [:upload_block_size, 0, 1 << 32 - 1, false],
       [:upload_recorder_upload_block_lifetime, 0, 1 << 64 - 1, true],
       [:upload_threshold, 0, 1 << 32 - 1, false],
       [:upload_token_lifetime, 0, 1 << 64 - 1, true],
       [:uplog_file_max_size, 0, 1 << 32 - 1, false],
       [:uplog_file_upload_threshold, 0, 1 << 32 - 1, false]].each do |method, min_value, max_value, is_time|
        define_method(method) do |arg|
          arg = Utils::Duration.new(arg) if is_time && arg.is_a?(Hash)
          arg = arg.to_i
          raise RangeError, "#{arg} is out of range" if arg > max_value || arg < min_value
          @builder.public_send(method, arg)
          self
        end
        alias_method :"#{method}=", method
      end

      # @!visibility private
      HTTPRequestBeforeActionHandler = proc do |request, err, idx|
        begin
          handler = CallbackData.get(idx)
          wrap_action_handler(err) do
            handler.call(HTTP::Request::send(:new, Bindings::HTTPRequest.new(request)))
          end
        rescue Exception => e
          Config::CallbackExceptionHandler.call(e)
        end
      end

      # @!visibility private
      HTTPRequestAfterActionHandler = proc do |request, response, err, idx|
        begin
          handler = CallbackData.get(idx)
          wrap_action_handler(err) do
            handler.call(
              HTTP::Request::send(:new, Bindings::HTTPRequest.new(request)),
              HTTP::Response::send(:new, Bindings::HTTPResponse.new(response)),
            )
          end
        rescue Exception => e
          Config::CallbackExceptionHandler.call(e)
        end
      end

      # @!visibility private
      HTTPRequestFinalHandler = proc do |request, response, err, idx|
        http_request = Bindings::HTTPRequest.new(request)
        idx = http_request.get_custom_data
        CallbackData.delete(idx) if idx
      end
      private_constant :HTTPRequestBeforeActionHandler, :HTTPRequestAfterActionHandler, :HTTPRequestFinalHandler

      # @!visibility private
      def self.wrap_action_handler(err)
        begin
          yield
        rescue Error::IOHandlerError => e
          err[:error] = QiniuNg::Bindings::CoreFFI::qiniu_ng_err_io_error_new(e.cause.message)
          err[:retry_kind] = e.retry_kind
          err[:is_retry_safe] = e.is_retry_safe?
        rescue Error::OSHandlerError => e
          err[:error] = QiniuNg::Bindings::CoreFFI::qiniu_ng_err_os_error_new(e.cause.errno)
          err[:retry_kind] = e.retry_kind
          err[:is_retry_safe] = e.is_retry_safe?
        rescue Error::UnexpectedRedirectHandlerError => e
          err[:error] = QiniuNg::Bindings::CoreFFI::qiniu_ng_err_unexpected_redirect_error_new
          err[:retry_kind] = e.retry_kind
          err[:is_retry_safe] = e.is_retry_safe?
        rescue Error::UserCancelledHandlerError => e
          err[:error] = QiniuNg::Bindings::CoreFFI::qiniu_ng_err_user_canceled_error_new
          err[:retry_kind] = e.retry_kind
          err[:is_retry_safe] = e.is_retry_safe?
        rescue Error::JSONHandlerError => e
          err[:error] = QiniuNg::Bindings::CoreFFI::qiniu_ng_err_json_error_new(e.cause.message)
          err[:retry_kind] = e.retry_kind
          err[:is_retry_safe] = e.is_retry_safe?
        rescue Error::ResponseStatusCodeHandlerError => e
          err[:error] = QiniuNg::Bindings::CoreFFI::qiniu_ng_err_response_status_code_error_new(e.cause.code, e.cause.message)
          err[:retry_kind] = e.retry_kind
          err[:is_retry_safe] = e.is_retry_safe?
        rescue Error::CurlHandlerError => e
          err[:error] = QiniuNg::Bindings::CoreFFI::qiniu_ng_err_curl_error_new(e.cause.curl_code, e.cause.original_error_kind)
          err[:retry_kind] = e.retry_kind
          err[:is_retry_safe] = e.is_retry_safe?
        end
      end
      private_class_method :wrap_action_handler
    end
  end
end
