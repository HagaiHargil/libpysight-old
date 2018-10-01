import libpysight

a = libpysight.read_binary_lst_u16('Mouse_LPT_189kHz_62p_Penetrating_arteries_FOV2_20x_Zoom_512lines_512px_400um_higher_than_Nominal_depth_800nm_039.lst', 
                                   1565, 82048, '43', [0, 1, 0, 0, 0, 1])
print(a['start'])
