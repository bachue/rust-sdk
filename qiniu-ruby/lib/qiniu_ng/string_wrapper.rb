# frozen_string_literal: true

module QiniuNg
  # @!visibility private
  class StringWrapper < String
    def initialize(str)
      @str = str
      super(@str.get_cstr)
    end

    # @!visibility private
    def inspect
      @str.get_cstr.inspect
    end
  end
end
