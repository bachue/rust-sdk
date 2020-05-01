require 'json'
require 'securerandom'
require 'tempfile'
require 'concurrent-ruby'

RSpec.describe QiniuNg::Storage::Uploader do
  context '#upload_file' do
    it 'should upload file by io' do
      uploader = QiniuNg::Storage::Uploader.create
      credential = QiniuNg::Credential.create(ENV['access_key'], ENV['secret_key'])
      client = QiniuNg::Client.create(credential: credential)
      bucket = client.bucket(ENV['upload_bucket'])
      Tempfile.create('测试', encoding: 'ascii-8bit') do |file|
        4.times { file.write(SecureRandom.random_bytes(rand(1 << 25))) }
        file.rewind

        key = "测试-#{Time.now.to_i}-#{rand(2**64 - 1)}"
        object = bucket.object(key)

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
        response = uploader.upload_file(file, credential: credential,
                                              bucket_name: ENV['upload_bucket'],
                                              key: key,
                                              on_uploading_progress: on_uploading_progress)
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

    it 'should upload customized io' do
      client = QiniuNg::Client.create(access_key: ENV['access_key'], secret_key: ENV['secret_key'])
      bucket = client.bucket(ENV['upload_bucket'])
      upload_token = QiniuNg::Storage::Uploader::UploadPolicy::Builder.new_for_bucket(ENV['upload_bucket'])
                                                                      .return_body(%[{"key":"$(key)","hash":"$(etag)","fsize":$(fsize),"bucket":"$(bucket)","name":"$(x:name)"}])
                                                                      .build_token(access_key: ENV['access_key'], secret_key: ENV['secret_key'])
      uploader = QiniuNg::Storage::Uploader.create
      key = "测试-#{Time.now.to_i}-#{rand(2**64 - 1)}"
      object = bucket.object(key)

      io = StringIO.new SecureRandom.random_bytes(1 << 24)
      etag = QiniuNg::Utils::Etag.from_io(io)
      io.rewind

      err = Concurrent::AtomicReference.new
      last_uploaded, mutex, io_size = -1, Mutex.new, io.size
      on_uploading_progress = ->(uploaded, total) do
                                begin
                                  expect(total).to eq io_size
                                  expect(uploaded <= total).to be true
                                  mutex.synchronize do
                                    last_uploaded = [last_uploaded, uploaded].max
                                  end
                                rescue Exception => e
                                  err.set(e)
                                end
                              end
      GC.start
      response = uploader.upload_io(io, upload_token: upload_token,
                                        key: key,
                                        file_name: key,
                                        vars: { 'name': key },
                                        on_uploading_progress: on_uploading_progress)
      GC.start
      expect(response.hash).to eq(etag)
      expect(response.key).to eq(key)
      expect(response.fsize).to eq(1 << 24)
      expect(response.bucket).to eq(ENV['upload_bucket'])
      expect(response.name).to eq(key)
      j = JSON.load response.as_json
      expect(j['hash']).to eq(etag)
      expect(j['key']).to eq(key)
      expect(j['fsize']).to eq(1 << 24)
      expect(j['bucket']).to eq(ENV['upload_bucket'])
      expect(j['name']).to eq(key)
      expect(err.get).to be_nil
      expect(last_uploaded).to eq io_size

      stat = object.stat
      expect(stat.size).to eq(1 << 24)
      expect(stat.hash).to eq(etag)
      expect(Time.now).to be_within(30).of(stat.uploaded_at)

      object.delete!
    end
  end

  context '#upload_file_path' do
    it 'should upload file by path' do
      uploader = QiniuNg::Storage::Uploader.create
      credential = QiniuNg::Credential.create(ENV['access_key'], ENV['secret_key'])
      client = QiniuNg::Client.create credential: credential
      bucket = client.bucket(ENV['upload_bucket'])
      Tempfile.create('测试', encoding: 'ascii-8bit') do |file|
        4.times { file.write(SecureRandom.random_bytes(rand(1 << 25))) }
        file.rewind
        etag = QiniuNg::Utils::Etag.from_io(file)
        key = "测试-#{Time.now.to_i}-#{rand(2**64 - 1)}"
        object = bucket.object(key)

        err = Concurrent::AtomicReference.new
        last_uploaded, mutex, file_size = -1, Mutex.new, file.size
        on_uploading_progress = ->(uploaded, total) do
                                  begin
                                    expect(total >= file_size).to be true
                                    expect(uploaded <= total).to be true
                                    mutex.synchronize do
                                      last_uploaded = [last_uploaded, uploaded].max
                                    end
                                  rescue Exception => e
                                    err.set(e)
                                  end
                                end

        response = uploader.upload_file_path(file.path, bucket_name: ENV['upload_bucket'],
                                                        credential: credential,
                                                        key: key,
                                                        on_uploading_progress: on_uploading_progress)
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
