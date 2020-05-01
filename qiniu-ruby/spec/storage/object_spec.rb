require 'json'
require 'securerandom'
require 'tempfile'
require 'concurrent-ruby'
require 'open-uri'

RSpec.describe QiniuNg::Storage::Object do
  before do
    @client = QiniuNg::Client.create access_key: ENV['access_key'], secret_key: ENV['secret_key']
  end

  context 'Get Object Info' do
    it 'can generate url of the object' do
      object = @client.bucket(ENV['public_bucket']).object('file')
      url = object.url lifetime: QiniuNg::Utils::Duration.new(hour: 1)
      expect(url).to be_end_with('/file')

      object = @client.bucket(ENV['private_bucket']).object('file')
      url = object.url deadline: Time.now
      expect(url).to include('/file?e=')
    end

    it 'can generate public url of the object' do
      object = @client.bucket(ENV['public_bucket']).object('file')
      url = object.public_url
      expect(url).to be_end_with('/file')

      object = @client.bucket(ENV['private_bucket']).object('file')
      url = object.public_url
      expect(url).to be_end_with('/file')
    end

    it 'can generate private url of the object' do
      object = @client.bucket(ENV['public_bucket']).object('file')
      url = object.private_url lifetime: QiniuNg::Utils::Duration.new(hour: 1)
      expect(url).to include('/file?e=')

      object = @client.bucket(ENV['private_bucket']).object('file')
      url = object.private_url deadline: Time.now
      expect(url).to include('/file?e=')
    end

    it 'can get info of the object' do
      Tempfile.create('测试', encoding: 'ascii-8bit') do |file|
        file_size = rand(1 << 22)
        file.write(SecureRandom.random_bytes(file_size))
        file.rewind

        key = "测试-#{Time.now.to_i}-#{rand(2**64 - 1)}"
        object = @client.bucket(ENV['upload_bucket']).object(key)

        response = object.upload_file(file)
        etag = response.hash

        header_info = object.head
        expect(header_info.etag).to eq etag.inspect
        expect(header_info.size).to eq file_size

        url = object.url lifetime: QiniuNg::Utils::Duration.new(hour: 1)
        URI.parse(url).open do |resp|
          expect(resp.size).to eq(file_size)
          expect(QiniuNg::Utils::Etag.from_io(resp)).to eq etag
        end
      end
    end
  end

  context '#upload_file' do
    it 'could upload file directly' do
      Tempfile.create('测试', encoding: 'ascii-8bit') do |file|
        4.times { file.write(SecureRandom.random_bytes(rand(1 << 25))) }
        file.rewind

        key = "测试-#{Time.now.to_i}-#{rand(2**64 - 1)}"
        object = @client.bucket(ENV['upload_bucket']).object(key)

        err = Concurrent::AtomicReference.new
        last_uploaded, mutex, file_size = -1, Mutex.new, file.size
        on_uploading_progress = ->(uploaded, total) do
                                  begin
                                    expect(total).to eq file_size
                                    expect(uploaded <= total).to be true
                                    mutex.synchronize do
                                      last_uploaded = [last_uploaded, uploaded].max
                                    end
                                  rescue Exception => e
                                    err.set(e)
                                  end
                                end

        etag = QiniuNg::Utils::Etag.from_io(file)
        file.rewind

        GC.start
        response = object.upload_file(file, on_uploading_progress: on_uploading_progress)
        GC.start
        expect(response.hash).to eq(etag)
        expect(response.key).to eq(key)
        j = JSON.load response.as_json
        expect(j['hash']).to eq(etag)
        expect(j['key']).to eq(key)
        expect(err.get).to be_nil
        expect(last_uploaded).to eq file_size

        stat = object.stat
        expect(stat.size).to eq(file.size)
        expect(stat.hash).to eq(etag)
        expect(Time.now).to be_within(30).of(stat.uploaded_at)

        object.delete!
      end
    end
  end

  context '#upload_file_path' do
    it 'could upload file directly' do
      Tempfile.create('测试', encoding: 'ascii-8bit') do |file|
        4.times { file.write(SecureRandom.random_bytes(rand(1 << 25))) }
        file.rewind
        etag = QiniuNg::Utils::Etag.from_io(file)
        file.rewind
        key = "测试-#{Time.now.to_i}-#{rand(2**64 - 1)}"
        object = @client.bucket(ENV['upload_bucket']).object(key)

        err = Concurrent::AtomicReference.new
        last_uploaded, mutex, file_size = -1, Mutex.new, file.size
        on_uploading_progress = ->(uploaded, total) do
                                  begin
                                    expect(total).to eq(file_size)
                                    expect(uploaded <= total).to be true
                                    mutex.synchronize do
                                      last_uploaded = [last_uploaded, uploaded].max
                                    end
                                  rescue Exception => e
                                    err.set(e)
                                  end
                                end

        response = object.upload_file_path(file.path, on_uploading_progress: on_uploading_progress)
        expect(response.hash).to eq(etag)
        expect(response.key).to eq(key)
        j = JSON.load response.as_json
        expect(j['hash']).to eq(etag)
        expect(j['key']).to eq(key)
        expect(err.get).to be_nil
        expect(last_uploaded).to eq file_size

        stat = object.stat
        expect(stat.size).to eq(file.size)
        expect(stat.hash).to eq(etag)
        expect(Time.now).to be_within(30).of(stat.uploaded_at)

        object.delete!
      end
    end
  end
end
