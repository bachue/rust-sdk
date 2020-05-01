require 'json'
require 'securerandom'
require 'tempfile'
require 'concurrent-ruby'

RSpec.describe QiniuNg::Storage::Uploader::BatchUploader do
  if RUBY_ENGINE != 'jruby'
    context '#upload_file' do
      it 'should upload files by io' do
        credential = QiniuNg::Credential.create(ENV['access_key'], ENV['secret_key'])
        batch_uploader = QiniuNg::Storage::Uploader.create.batch_uploader(bucket_name: ENV['upload_bucket'], credential: credential)
        batch_uploader.thread_pool_size = 8
        completed = Concurrent::AtomicFixnum.new
        errref = Concurrent::AtomicReference.new
        infos = []
        8.times do |idx|
          tempfile = Tempfile.create('测试', encoding: 'ascii-8bit')
          tempfile.write(SecureRandom.random_bytes(rand(1 << 23)))
          tempfile.rewind
          etag = QiniuNg::Utils::Etag.from_io(tempfile)
          tempfile.rewind
          key = "测试-#{idx}-#{Time.now.to_i}"
          infos.push(key: key, size: tempfile.size, etag: etag)
          batch_uploader.upload_file(tempfile, key: key) do |response, err|
            begin
              expect(err).to be_nil
              expect(response).not_to be_nil
              expect(response.hash).to eq etag
              expect(response.key).to eq key
              completed.increment
            rescue Exception => e
              errref.set(e)
            end
          end
        end

        GC.start
        batch_uploader.start
        GC.start

        expect(errref.get).to be_nil
        expect(completed.value).to eq 8

        client = QiniuNg::Client.create(credential: credential)
        bucket = client.bucket(ENV['upload_bucket'])

        infos.each do |info|
          object = bucket.object(info[:key])
          stat = object.stat
          expect(stat.size).to eq(info[:size])
          expect(stat.hash).to eq(info[:etag])
          expect(Time.now).to be_within(240).of(stat.uploaded_at)

          object.delete!
        end
      end

      it 'should upload files by path' do
        credential = QiniuNg::Credential.create(ENV['access_key'], ENV['secret_key'])
        batch_uploader = QiniuNg::Storage::Uploader.create.batch_uploader(bucket_name: ENV['upload_bucket'], credential: credential)
        batch_uploader.thread_pool_size = 8
        completed = Concurrent::AtomicFixnum.new
        errref = Concurrent::AtomicReference.new
        infos = []
        8.times do |idx|
          tempfile = Tempfile.create('测试', encoding: 'ascii-8bit')
          file_size = rand(1 << 23)
          tempfile.write(SecureRandom.random_bytes(file_size))
          tempfile.rewind
          etag = QiniuNg::Utils::Etag.from_io(tempfile)
          tempfile.rewind
          key = "测试-#{idx}-#{Time.now.to_i}"
          infos.push(key: key, size: tempfile.size, etag: etag)
          on_uploading_progress = ->(uploaded, total) do
                                    expect(total >= file_size).to be true
                                    expect(uploaded <= total).to be true
                                  end
          batch_uploader.upload_file_path(tempfile.path, key: key, on_uploading_progress: on_uploading_progress) do |response, err|
            begin
              expect(err).to be_nil
              expect(response).not_to be_nil
              expect(response.hash).to eq etag
              expect(response.key).to eq key
              completed.increment
            rescue Exception => e
              errref.set(e)
            end
          end
        end
        GC.start
        batch_uploader.start
        GC.start
        expect(errref.get).to be_nil
        expect(completed.value).to eq 8

        client = QiniuNg::Client.create(credential: credential)
        bucket = client.bucket(ENV['upload_bucket'])

        infos.each do |info|
          object = bucket.object(info[:key])
          stat = object.stat
          expect(stat.size).to eq(info[:size])
          expect(stat.hash).to eq(info[:etag])
          expect(Time.now).to be_within(240).of(stat.uploaded_at)

          object.delete!
        end
      end
    end
  end
end
