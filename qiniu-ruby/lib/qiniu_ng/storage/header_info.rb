# frozen_string_literal: true

module QiniuNg
  module Storage
    # Header 信息
    #
    # 用于封装访问下载 URL 时获得的 Header 信息
    class HeaderInfo
      # @!visibility private
      def initialize(header_info_ffi)
        @header_info = header_info_ffi
        @cache = {}
      end
      private_class_method :new

      # 获取对象 MIME 类型
      # @return [String] 返回对象 MIME 类型
      def content_type
        @cache[:content_type] ||= @header_info.get_content_type
        return nil if @cache[:content_type].is_null
        @cache[:content_type].get_cstr
      end

      # 获取对象 Etag 校验和
      # @return [String] 返回对象 Etag 校验和
      def etag
        @cache[:etag] ||= @header_info.get_etag
        return nil if @cache[:etag].is_null
        @cache[:etag].get_cstr
      end

      # 获取对象元数据
      # @return [String] 返回对象元数据
      def metadata
        metadata = {}
        handler = ->(name, value, _) do
          metadata[name.force_encoding(Encoding::UTF_8)] = value.force_encoding(Encoding::UTF_8)
          true
        end
        @header_info.get_metadata.each_entry(handler, nil)
        metadata
      end

      # 获取对象尺寸
      # @return [Integer] 返回对象尺寸
      def size
        @cache[:size] ||= @header_info.get_size
        return nil if @cache[:size].is_null
        @cache[:size].get_cstr.to_i
      end
    end
  end
end
