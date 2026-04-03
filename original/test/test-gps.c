/* test-gps.c
 *
 * Creates ExifEntries for the GPS ifd and initializes them.
 * Checks for formats and component counts.
 *
 * Copyright 2020 Heiko Lewin <hlewin@gmx.de>
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, write to the
 * Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA  02110-1301  USA.
 */

#include <libexif/exif-data.h>
#include "test-public-api.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>

/*
 * List of tags to test, 
 */
const uint16_t test_tags [] = {
 EXIF_TAG_GPS_VERSION_ID,
 EXIF_TAG_GPS_LATITUDE_REF,
 EXIF_TAG_GPS_LATITUDE,
 EXIF_TAG_GPS_LONGITUDE_REF,     
 EXIF_TAG_GPS_LONGITUDE,         
 EXIF_TAG_GPS_ALTITUDE_REF,      
 EXIF_TAG_GPS_ALTITUDE,          
 EXIF_TAG_GPS_TIME_STAMP,        
 EXIF_TAG_GPS_SATELLITES,        
 EXIF_TAG_GPS_STATUS,            
 EXIF_TAG_GPS_MEASURE_MODE,      
 EXIF_TAG_GPS_DOP,               
 EXIF_TAG_GPS_SPEED_REF,         
 EXIF_TAG_GPS_SPEED,             
 EXIF_TAG_GPS_TRACK_REF,         
 EXIF_TAG_GPS_TRACK,             
 EXIF_TAG_GPS_IMG_DIRECTION_REF, 
 EXIF_TAG_GPS_IMG_DIRECTION,     
 EXIF_TAG_GPS_MAP_DATUM,         
 EXIF_TAG_GPS_DEST_LATITUDE_REF, 
 EXIF_TAG_GPS_DEST_LATITUDE,     
 EXIF_TAG_GPS_DEST_LONGITUDE_REF, 
 EXIF_TAG_GPS_DEST_LONGITUDE,     
 EXIF_TAG_GPS_DEST_BEARING_REF,   
 EXIF_TAG_GPS_DEST_BEARING,       
 EXIF_TAG_GPS_DEST_DISTANCE_REF,  
 EXIF_TAG_GPS_DEST_DISTANCE,      
 EXIF_TAG_GPS_PROCESSING_METHOD,  
 EXIF_TAG_GPS_AREA_INFORMATION,   
 EXIF_TAG_GPS_DATE_STAMP,         
 EXIF_TAG_GPS_DIFFERENTIAL,
 EXIF_TAG_GPS_H_POSITIONING_ERROR,
 0xFFFFu
};


/*
 * Verify that the entry is properly initialized.
 */
static int check_entry_format(ExifEntry *e)
{
	TestEntryInfo info;

	if (test_entry_info_from_dump(e, &info)) {
		fprintf(stderr, "check_entry_format: Could not read entry via public API dump\n");
		return 1;
	}
	if (info.tag > EXIF_TAG_GPS_H_POSITIONING_ERROR) {
		/* Unknown tags should get EXIF_FORMAT_UNDEFINED and no payload. */
		if (info.format != EXIF_FORMAT_UNDEFINED || info.size || info.components) {
		    fprintf(stderr, "check_entry_format: Unknown tag not handled correctly (tag=%x)\n", info.tag);
		    return 1;
		}
		return 0;
	}
	switch(info.format) {
	case EXIF_FORMAT_UNDEFINED:
	case EXIF_FORMAT_ASCII:
		/* entries with ASCII or UNDEFINED format do not necessarily need to have the component count set.
		   only check here is, if component count is set, the size should match the count */
		if(info.size != info.components) {
			fprintf (stderr, "check_entry_format: Entry has bad component count or size (tag=%x)\n", info.tag);
			return 1;
		}
		break;
		
	default:
		/* All other formats should have a nonzero component count. */
		if(!info.components) {
			fprintf (stderr, "check_entry_format: Entry should have component count set (tag=%x)\n", info.tag);
			return 1;
		}
		return 0;	
	}

	/* If a value is present the size should be set to the right value */	
	if(info.size && info.size != info.components * exif_format_get_size((ExifFormat)info.format)) {
		fprintf (stderr, "check_entry_format: Entry has bad size (tag=%x)\n", info.tag);
		return 1;	
	}
	return 0;
}

int
main ()
{
	size_t i;
	ExifData *data = NULL;
	ExifEntry *e = NULL;
	ExifContent *gps_content;

	data = exif_data_new ();
	if (!data) {
		fprintf (stderr, "Error running exif_data_new()\n");
		goto ERROR_EXIT;
	}
	gps_content = test_find_ifd_content(data, EXIF_IFD_GPS);
	if (!gps_content) {
		fprintf (stderr, "Error finding GPS IFD\n");
		goto ERROR_EXIT;
	}

	/* Run tests */
	for (i=0; i < sizeof(test_tags)/sizeof(test_tags[0]); ++i) {
		e = exif_entry_new ();
		if (!e) {
			fprintf (stderr, "Error running exif_entry_new()\n");
			goto ERROR_EXIT;
		}		
		exif_content_add_entry(gps_content, e);
		exif_entry_initialize (e, (ExifTag)test_tags[i]);
		if(check_entry_format(e)) goto ERROR_EXIT;
		exif_content_remove_entry (gps_content, e);
		exif_entry_unref (e);
		e = NULL;
	}
	exif_data_unref(data);
	return 0;
ERROR_EXIT:
	exif_entry_unref (e);
	exif_data_unref (data);
	exit(EXIT_FAILURE);
}
