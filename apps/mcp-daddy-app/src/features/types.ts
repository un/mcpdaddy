export type UpstreamStatus = 'healthy' | 'unhealthy' | 'stopped';

export type Upstream = {
  id: string;
  displayName: string;
  status: UpstreamStatus;
};

export type ExposureMode = 'full' | 'compact';

export type ClientProfile = {
  id: string;
  displayName: string;
  exposureMode: ExposureMode;
  allowedUpstreamIds: string[];
};
