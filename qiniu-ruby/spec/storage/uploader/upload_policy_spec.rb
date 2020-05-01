require 'json'

RSpec.describe QiniuNg::Storage::Uploader::UploadPolicy do
  context '#new_for_bucket' do
    it 'should create upload policy for the bucket' do
      upload_policy = QiniuNg::Storage::Uploader::UploadPolicy.new_for_bucket(ENV['public_bucket'])
      expect(upload_policy.bucket).to eq(ENV['public_bucket'])
      expect(upload_policy.key).to be_nil
      expect(upload_policy.prefixal_object_key?).to be false
      expect(upload_policy.token_lifetime.to_i).to be_within(5).of 3600
      expect(upload_policy.token_deadline.to_i).to be_within(5).of(Time.now.to_i + 3600)
      j = JSON.load(upload_policy.as_json)
      expect(j['scope']).to eq ENV['public_bucket']
      expect(j['deadline']).to be_within(5).of(Time.now.to_i + 3600)
    end

    it 'should create upload policy for the object' do
      upload_policy = QiniuNg::Storage::Uploader::UploadPolicy.new_for_object(ENV['public_bucket'], 'test-key')
      expect(upload_policy.bucket).to eq ENV['public_bucket']
      expect(upload_policy.key).to eq 'test-key'
      expect(upload_policy.prefixal_object_key?).to be false
      expect(upload_policy.token_lifetime.to_i).to be_within(5).of 3600
      expect(upload_policy.token_deadline.to_i).to be_within(5).of(Time.now.to_i + 3600)
      j = JSON.load(upload_policy.as_json)
      expect(j['scope']).to eq "#{ENV['public_bucket']}:test-key"
      expect(j['deadline']).to be_within(5).of(Time.now.to_i + 3600)
    end

    it 'should create upload policy for the objects with the same prefix' do
      config = QiniuNg::Config.create(upload_token_lifetime: QiniuNg::Utils::Duration.new(hours: 2))
      upload_policy = QiniuNg::Storage::Uploader::UploadPolicy.new_for_objects_with_prefix(ENV['public_bucket'], 'test-key', config: config)
      expect(upload_policy.bucket).to eq ENV['public_bucket']
      expect(upload_policy.key).to eq 'test-key'
      expect(upload_policy.prefixal_object_key?).to be true
      expect(upload_policy.token_lifetime.to_i).to be_within(5).of 7200
      expect(upload_policy.token_deadline.to_i).to be_within(5).of(Time.now.to_i + 7200)
      j = JSON.load(upload_policy.as_json)
      expect(j['scope']).to eq "#{ENV['public_bucket']}:test-key"
      expect(j['deadline']).to be_within(5).of(Time.now.to_i + 7200)
    end

    it 'should set token lifetime for the upload policy' do
      upload_policy = QiniuNg::Storage::Uploader::UploadPolicy::Builder.new_for_bucket(ENV['public_bucket'])
                                                                       .token_lifetime(hours: 2)
                                                                       .build!
      expect(upload_policy.token_lifetime.to_i).to be_within(5).of 7200
      expect(upload_policy.token_deadline.to_i).to be_within(5).of(Time.now.to_i + 7200)
      j = JSON.load(upload_policy.as_json)
      expect(j['scope']).to eq ENV['public_bucket']
      expect(j['deadline']).to be_within(5).of(Time.now.to_i + 7200)
    end

    it 'should set object deadline for the upload policy' do
      upload_policy = QiniuNg::Storage::Uploader::UploadPolicy::Builder.new_for_bucket(ENV['public_bucket'])
                                                                       .object_deadline(Time.now + 86400)
                                                                       .build!
      expect(upload_policy.object_lifetime.to_i).to be_within(5).of 86400
      expect(upload_policy.object_deadline.to_i).to be_within(5).of(Time.now.to_i + 86400)

      upload_policy = QiniuNg::Storage::Uploader::UploadPolicy::Builder.new_for_bucket(ENV['public_bucket'])
                                                                       .object_lifetime(QiniuNg::Utils::Duration.new(day: 1))
                                                                       .build!
      expect(upload_policy.object_lifetime.to_i).to be_within(5).of 86400
      expect(upload_policy.object_deadline.to_i).to be_within(5).of(Time.now.to_i + 86400)

      upload_policy = QiniuNg::Storage::Uploader::UploadPolicy::Builder.new_for_bucket(ENV['public_bucket'])
                                                                       .object_lifetime(day: 1)
                                                                       .build!
      expect(upload_policy.object_lifetime.to_i).to be_within(5).of 86400
      expect(upload_policy.object_deadline.to_i).to be_within(5).of(Time.now.to_i + 86400)
    end

    it 'could set file type for the upload policy' do
      upload_policy = QiniuNg::Storage::Uploader::UploadPolicy::Builder.new_for_bucket(ENV['public_bucket'])
                                                                       .use_normal_storage
                                                                       .build!
      expect(upload_policy.normal_storage_used?).to be true
      expect(upload_policy.infrequent_storage_used?).to be false

      upload_policy = QiniuNg::Storage::Uploader::UploadPolicy::Builder.new_for_bucket(ENV['public_bucket'])
                                                                       .use_infrequent_storage
                                                                       .build!
      expect(upload_policy.normal_storage_used?).to be false
      expect(upload_policy.infrequent_storage_used?).to be true

      j = JSON.load(upload_policy.as_json)
      expect(j['fileType']).to eq 1
    end

    it 'could set file type for the upload policy' do
      upload_policy = QiniuNg::Storage::Uploader::UploadPolicy::Builder.new_for_object(ENV['public_bucket'], 'test-key')
                                                                       .overwritable
                                                                       .build!
      expect(upload_policy.overwritable?).to be true
      expect(upload_policy.insert_only?).to be false

      upload_policy = QiniuNg::Storage::Uploader::UploadPolicy::Builder.new_for_object(ENV['public_bucket'], 'test-key')
                                                                       .insert_only
                                                                       .build!
      expect(upload_policy.insert_only?).to be true
      expect(upload_policy.overwritable?).to be false

      j = JSON.load(upload_policy.as_json)
      expect(j['insertOnly']).to eq 1
    end

    it 'could set file size limitation for the upload policy' do
      upload_policy = QiniuNg::Storage::Uploader::UploadPolicy::Builder.new_for_bucket(ENV['public_bucket'])
                                                                       .file_size_limitation(nil, 1024)
                                                                       .build!
      expect(upload_policy.file_size_limitation).to eq([nil, 1024])

      upload_policy = QiniuNg::Storage::Uploader::UploadPolicy::Builder.new_for_bucket(ENV['public_bucket'])
                                                                       .file_size_limitation(1024, nil)
                                                                       .build!
      expect(upload_policy.file_size_limitation).to eq([1024, nil])

      upload_policy = QiniuNg::Storage::Uploader::UploadPolicy::Builder.new_for_bucket(ENV['public_bucket'])
                                                                       .file_size_limitation(1024, 102400)
                                                                       .build!
      expect(upload_policy.file_size_limitation).to eq([1024, 102400])

      j = JSON.load(upload_policy.as_json)
      expect(j['fsizeMin']).to eq 1024
      expect(j['fsizeLimit']).to eq 102400
    end

    it 'could set return attributes for the upload policy' do
      upload_policy = QiniuNg::Storage::Uploader::UploadPolicy.new_for_bucket(ENV['public_bucket'])
      expect(upload_policy.return_url).to be_nil
      expect(upload_policy.return_body).to be_nil

      upload_policy = QiniuNg::Storage::Uploader::UploadPolicy::Builder.new_for_bucket(ENV['public_bucket'])
                                                                       .return_url('http://qiniu.com')
                                                                       .build!
      expect(upload_policy.return_url).to eq 'http://qiniu.com'
      expect(upload_policy.return_body).to be_nil

      upload_policy = QiniuNg::Storage::Uploader::UploadPolicy::Builder.new_for_bucket(ENV['public_bucket'])
                                                                       .return_body('<h1>Qiniu</h1>')
                                                                       .build!
      expect(upload_policy.return_url).to be_nil
      expect(upload_policy.return_body).to eq '<h1>Qiniu</h1>'

      upload_policy = QiniuNg::Storage::Uploader::UploadPolicy::Builder.new_for_bucket(ENV['public_bucket'])
                                                                       .return_url('http://qiniu.com')
                                                                       .return_body('<h1>Qiniu</h1>')
                                                                       .build!
      expect(upload_policy.return_url).to eq 'http://qiniu.com'
      expect(upload_policy.return_body).to eq '<h1>Qiniu</h1>'
    end

    it 'could set callback attributes for the upload policy' do
      upload_policy = QiniuNg::Storage::Uploader::UploadPolicy.new_for_bucket(ENV['public_bucket'])
      expect(upload_policy.callback_body).to be_nil
      expect(upload_policy.callback_body_type).to be_nil
      expect(upload_policy.callback_host).to be_nil
      expect(upload_policy.callback_urls).to be_empty

      upload_policy = QiniuNg::Storage::Uploader::UploadPolicy::Builder.new_for_bucket(ENV['public_bucket'])
                                                                       .callback('https://www.qiniu.com', body: 'testbody')
                                                                       .build!
      expect(upload_policy.callback_body).to eq 'testbody'
      expect(upload_policy.callback_body_type).to be_nil
      expect(upload_policy.callback_host).to be_nil
      expect(upload_policy.callback_urls).to contain_exactly('https://www.qiniu.com')

      upload_policy = QiniuNg::Storage::Uploader::UploadPolicy::Builder.new_for_bucket(ENV['public_bucket'])
                                                                       .callback('https://www.qiniu.com', host: 'qiniu.com', body: 'testbody', body_type: 'text/plain')
                                                                       .build!
      expect(upload_policy.callback_body).to eq 'testbody'
      expect(upload_policy.callback_body_type).to eq 'text/plain'
      expect(upload_policy.callback_host).to eq 'qiniu.com'
      expect(upload_policy.callback_urls).to contain_exactly('https://www.qiniu.com')

      upload_policy = QiniuNg::Storage::Uploader::UploadPolicy::Builder.new_for_bucket(ENV['public_bucket'])
                                                                       .callback(%w[https://www.qiniu.com https://www2.qiniu.com], host: 'qiniu.com', body: 'testbody', body_type: 'text/plain')
                                                                       .build!
      expect(upload_policy.callback_body).to eq 'testbody'
      expect(upload_policy.callback_body_type).to eq 'text/plain'
      expect(upload_policy.callback_host).to eq 'qiniu.com'
      expect(upload_policy.callback_urls).to contain_exactly('https://www.qiniu.com', 'https://www2.qiniu.com')
    end

    it 'could set mime_types for the upload policy' do
      upload_policy = QiniuNg::Storage::Uploader::UploadPolicy::Builder.new_for_bucket(ENV['public_bucket'])
                                                                       .mime_types('text/plain')
                                                                       .build!
      expect(upload_policy.mime_types).to contain_exactly('text/plain')

      upload_policy = QiniuNg::Storage::Uploader::UploadPolicy::Builder.new_for_bucket(ENV['public_bucket'])
                                                                       .mime_types(['text/plain', 'text/html'])
                                                                       .build!
      expect(upload_policy.mime_types).to contain_exactly('text/plain', 'text/html')
    end

    it 'could set mime_detection for the upload policy' do
      upload_policy = QiniuNg::Storage::Uploader::UploadPolicy::Builder.new_for_bucket(ENV['public_bucket'])
                                                                       .disable_mime_detection
                                                                       .build!
      expect(upload_policy.mime_detection_enabled?).to be false

      upload_policy = QiniuNg::Storage::Uploader::UploadPolicy::Builder.new_for_bucket(ENV['public_bucket'])
                                                                       .enable_mime_detection
                                                                       .build!
      expect(upload_policy.mime_detection_enabled?).to be true
    end

    it 'could save key as specified for the upload policy' do
      upload_policy = QiniuNg::Storage::Uploader::UploadPolicy::Builder.new_for_bucket(ENV['public_bucket'])
                                                                       .save_as('force-key')
                                                                       .build!
      expect(upload_policy.save_key).to eq 'force-key'
      expect(upload_policy.save_key_forced?).to be false

      j = JSON.load(upload_policy.as_json)
      expect(j['saveKey']).to eq 'force-key'
      expect(j['forceSaveKey']).to be_nil

      upload_policy = QiniuNg::Storage::Uploader::UploadPolicy::Builder.new_for_bucket(ENV['public_bucket'])
                                                                       .save_as('force-key', force: true)
                                                                       .build!
      expect(upload_policy.save_key).to eq 'force-key'
      expect(upload_policy.save_key_forced?).to be true

      j = JSON.load(upload_policy.as_json)
      expect(j['saveKey']).to eq 'force-key'
      expect(j['forceSaveKey']).to be true
    end
  end
end
