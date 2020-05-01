# frozen_string_literal: true

module QiniuNg
  # 七牛 SDK 客户端
  #
  # 这里的客户端是针对七牛服务器而言，而并非指该结构体是运行在客户端应用程序上。
  # 实际上，该结构体由于会存储用户的 SecretKey，因此不推荐在客户端应用程序上使用，而应该只在服务器端应用程序上使用。
  #
  # 您可以通过该类作为入口调用到七牛大部分功能。
  class Client
    # @!visibility private
    def initialize(client_ffi)
      @client = client_ffi
    end
    private_class_method :new

    # 创建新的七牛客户端
    #
    # @example
    #   access_key = '[Qiniu Access Key]'
    #   secret_key = '[Qiniu Secret Key]'
    #   client = QiniuNg::Client.create access_key: access_key, secret_key: secret_key
    #
    # @param [String] access_key 七牛 Access Key，如果提供了 `credential`，则无需传入该参数
    # @param [String] secret_key 七牛 Secret Key，如果提供了 `credential`，则无需传入该参数
    # @param [Credential] credential 七牛认证信息，如果提供，将无需传入 `access_key` 和 `secret_key`
    # @param [Config] config 七牛客户端配置，默认将会创建默认配置
    # @return [Client] 返回新的客户端实例
    # @raise [ArgumentError] config 参数错误
    def self.create(access_key: nil, secret_key: nil, credential: nil, config: nil)
      raise ArgumentError, 'config must be instance of Config' unless config.nil? || config.is_a?(Config)
      client = if access_key && secret_key
                 if config.nil?
                   Bindings::Client.new_default(access_key.to_s, secret_key.to_s)
                 else
                   Bindings::Client.new!(access_key.to_s, secret_key.to_s, config.instance_variable_get(:@config))
                 end
               elsif credential
                 raise ArgumentError, 'credential must be instance of Credential' unless credential.nil? || credential.is_a?(Credential)
                 if config.nil?
                   Bindings::Client.new_default_from_credential(credential.instance_variable_get(:@credential))
                 else
                   Bindings::Client.new_from_credential(credential.instance_variable_get(:@credential), config.instance_variable_get(:@config))
                 end
               end
      new(client)
    end

    # 获取 Access Key
    # @return [String] 返回 Access Key
    def access_key
      @access_key ||= @client.get_access_key
      @access_key.get_cstr
    end

    # 获取 Secret Key
    # @return [String] 返回 Secret Key
    def secret_key
      @secret_key ||= @client.get_secret_key
      @secret_key.get_cstr
    end

    # 获取认证信息
    # @return [Credential] 返回认证信息
    def credential
      @credential ||= Credential.create(self.access_key, self.secret_key)
    end

    # 获取客户端配置
    # @return [Config] 返回客户端配置
    def config
      @config ||= Config.send(:new, @client.get_config)
    end

    # 创建上传管理器
    # @return [Uploader] 返回上传管理器
    def uploader
      Storage::Uploader.create(self.config)
    end

    # 为指定 Bucket 创建存储空间上传器
    # @param [String] bucket_name 存储空间名称
    # @param [Integer] thread_pool_size 上传线程池尺寸，默认使用默认的线程池策略
    # @return [BucketUploader] 返回存储空间上传器
    # @raise [ArgumentError] 参数错误
    def uploader_for(bucket_name, thread_pool_size: nil)
      self.uploader.bucket_uploader(bucket_name: bucket_name, access_key: self.access_key, thread_pool_size: thread_pool_size)
    end

    # 获取指定的存储空间实例
    # @param [String] bucket_name 存储空间名称
    # @param [Region,Symbol] region 存储空间区域，如果传入 nil 将使用懒加载自动检测。
    #                        可以接受 {Storage::Region} 实例或符号，对于传入符号的情况，如果是 `:auto_detect` 表示立即检测，而其他符号表示区域 ID
    # @param [Array<String>] domains 下载域名列表
    # @param [Boolean] auto_detect_domains 是否自动检测下载域名，如果是，将首先使用传入的 domains，如果无法使用，才会选择七牛存储的下载域名
    # @return [Storage::Bucket] 返回存储空间实例
    def bucket(bucket_name, region: nil, domains: [], auto_detect_domains: false)
      Storage::Bucket.send(:init, self, bucket_name, region, domains, auto_detect_domains)
    end

    # 列出所有存储空间名称
    # @return [Array<String>] 返回所有存储空间名称
    def bucket_names
      list = Error.wrap_ffi_function do
               Bindings::Storage.bucket_names(@client)
             end
      (0...list.len).map { |i| list.get(i) }
    end

    # 创建存储空间
    #
    # 在创建存储空间时，需要注意存储空间的名称必须遵守以下规则：
    # - 存储空间名称不允许重复，遇到冲突请更换名称。
    # - 名称由 3 ~ 63 个字符组成 ，可包含小写字母、数字和短划线，且必须以小写字母或者数字开头和结尾。
    #
    # @example
    #   client.create_bucket('[New Bucket Name]', :z0)
    #
    # @param [String] bucket_name 存储空间名称
    # @param [Symbol] region 区域 ID，公有云区域 ID 参考 [官方文档](https://developer.qiniu.com/kodo/manual/1671/region-endpoint)
    # @return [Storage::Bucket] 返回新的存储空间实例
    def create_bucket(bucket_name, region)
      region = region.id if region.is_a?(Storage::Region)
      region_id = case region.to_sym
                  when :z0 then :qiniu_ng_region_z0
                  when :z1 then :qiniu_ng_region_z1
                  when :z2 then :qiniu_ng_region_z2
                  when :as0 then :qiniu_ng_region_as0
                  when :na0 then :qiniu_ng_region_na0
                  else
                    region.to_s
                  end
      Error.wrap_ffi_function do
        if region_id.is_a?(Symbol)
          Bindings::Storage.create_bucket(@client, bucket_name.to_s, region_id)
        else
          Bindings::Storage.create_bucket_with_customized_region_id(@client, bucket_name.to_s, region_id)
        end
      end
      bucket(bucket_name.to_s)
    end

    # 删除存储空间
    #
    # 删除存储空间前务必保证存储空间里已经没有任何文件，否则删除将会失败
    #
    # @param [String] bucket_name 存储空间名称
    def drop_bucket(bucket_name)
      Error.wrap_ffi_function do
        Bindings::Storage.drop_bucket(@client, bucket_name.to_s)
      end
      nil
    end

    # @!visibility private
    def inspect
      "#<#{self.class.name}>"
    end
  end
end
